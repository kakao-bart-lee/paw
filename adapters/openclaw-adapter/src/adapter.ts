import WebSocket from 'ws';
import { randomUUID } from 'node:crypto';
import { decryptContent, encryptContent, looksLikeCiphertext } from './e2ee';
import {
  AgentResponse,
  AgentResponseMsg,
  AgentStreamFrame,
  E2eeConfig,
  InboundContext,
  MessageReceived,
} from './types';
import { ChannelHandler } from './channel';

export interface ChannelConfig {
  defaultChannel?: string;
}

export interface AdapterOptions {
  serverUrl?: string;
  channelConfig?: ChannelConfig;
  agentId?: string;
  defaultFormat?: 'plain' | 'markdown';
  e2ee?: E2eeConfig;
}

const PROTOCOL_VERSION = 1;

export function parseInboundContext(raw: string): InboundContext | null {
  try {
    const data = JSON.parse(raw) as unknown;
    if (!isInboundContext(data)) {
      return null;
    }
    return data;
  } catch {
    return null;
  }
}

export function serializeAgentResponse(
  conversationId: string,
  content: string,
  format: 'plain' | 'markdown' = 'markdown',
): AgentResponseMsg {
  return {
    v: PROTOCOL_VERSION,
    conversation_id: conversationId,
    content,
    format,
  };
}

export class OpenClawAdapter {
  private ws: WebSocket | null = null;
  private handlers: Map<string, ChannelHandler> = new Map();
  private readonly serverUrl: string;
  private readonly channelConfig: ChannelConfig;
  private readonly agentId: string;
  private readonly defaultFormat: 'plain' | 'markdown';
  private readonly e2eeConfig: E2eeConfig | null;

  constructor(private readonly token: string, options: AdapterOptions = {}) {
    this.serverUrl = (options.serverUrl ?? 'ws://localhost:3000').replace(/\/$/, '');
    this.channelConfig = options.channelConfig ?? {};
    this.agentId = options.agentId ?? '00000000-0000-0000-0000-000000000000';
    this.defaultFormat = options.defaultFormat ?? 'markdown';
    this.e2eeConfig = options.e2ee ?? null;
  }

  registerChannel(handler: ChannelHandler): this {
    this.handlers.set(handler.name, handler);
    return this;
  }

  async connect(): Promise<void> {
    const url = `${this.serverUrl}/agent/ws?token=${this.token}`;
    this.ws = new WebSocket(url);

    await new Promise<void>((resolve, reject) => {
      if (!this.ws) {
        reject(new Error('WebSocket not initialized'));
        return;
      }

      const ws = this.ws;
      ws.once('open', () => resolve());
      ws.once('error', (error) => reject(error));
      ws.on('message', async (payload) => {
        const raw = typeof payload === 'string' ? payload : payload.toString();
        const ctx = parseInboundContext(raw);
        if (!ctx) {
          return;
        }
        await this.routeInboundContext(ctx);
      });
    });
  }

  disconnect(): void {
    this.ws?.close();
    this.ws = null;
  }

  async routeInboundContext(ctx: InboundContext): Promise<void> {
    let routedContext = ctx;
    const inboundContent = ctx.message.content;

    if (this.e2eeConfig && looksLikeCiphertext(inboundContent)) {
      const decrypted = decryptContent(this.e2eeConfig.privateKey, inboundContent);
      if (decrypted !== null) {
        routedContext = {
          ...ctx,
          message: {
            ...ctx.message,
            content: decrypted,
          },
        };
      }
    }

    const handler = this.resolveHandler(routedContext);
    if (!handler || !this.ws) {
      return;
    }

    const response = await handler.handle(routedContext);
    if (typeof response === 'string') {
      let responseContent = response;
      if (this.e2eeConfig) {
        responseContent = encryptContent(this.e2eeConfig.publicKey, response);
      }

      const message: AgentResponse = serializeAgentResponse(
        routedContext.conversation_id,
        responseContent,
        this.defaultFormat,
      );
      this.ws.send(JSON.stringify(message));
      return;
    }

    const streamId = randomUUID();
    const streamStart: AgentStreamFrame = {
      type: 'stream_start',
      v: PROTOCOL_VERSION,
      conversation_id: routedContext.conversation_id,
      agent_id: this.agentId,
      stream_id: streamId,
    };
    this.ws.send(JSON.stringify(streamStart));

    for await (const delta of response) {
      const frame: AgentStreamFrame = {
        type: 'content_delta',
        v: PROTOCOL_VERSION,
        stream_id: streamId,
        delta,
      };
      this.ws.send(JSON.stringify(frame));
    }

    const endFrame: AgentStreamFrame = {
      type: 'stream_end',
      v: PROTOCOL_VERSION,
      stream_id: streamId,
      tokens: 0,
      duration_ms: 0,
    };
    this.ws.send(JSON.stringify(endFrame));
  }

  private resolveHandler(ctx: InboundContext): ChannelHandler | undefined {
    const explicitChannel = this.channelConfig.defaultChannel;
    if (explicitChannel) {
      return this.handlers.get(explicitChannel);
    }

    if (ctx.type && this.handlers.has(ctx.type)) {
      return this.handlers.get(ctx.type);
    }

    return this.handlers.values().next().value;
  }
}

function isInboundContext(data: unknown): data is InboundContext {
  if (!isRecord(data)) {
    return false;
  }

  return (
    data.v === PROTOCOL_VERSION &&
    typeof data.conversation_id === 'string' &&
    Array.isArray(data.recent_messages) &&
    isMessageReceived(data.message)
  );
}

function isMessageReceived(value: unknown): value is MessageReceived {
  if (!isRecord(value)) {
    return false;
  }

  return (
    value.v === PROTOCOL_VERSION &&
    value.type === 'message_received' &&
    typeof value.id === 'string' &&
    typeof value.conversation_id === 'string' &&
    typeof value.sender_id === 'string' &&
    typeof value.content === 'string' &&
    (value.format === 'plain' || value.format === 'markdown') &&
    typeof value.seq === 'number' &&
    typeof value.created_at === 'string'
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}
