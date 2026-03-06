import WebSocket from 'ws';
import {
  AgentOptions,
  ConversationContext,
  Message,
  MessageHandler,
  PROTOCOL_VERSION,
} from './types';
import { StreamingContextImpl } from './streaming';

const DEFAULT_SERVER_URL = 'ws://localhost:3000';

/**
 * PawAgent — TypeScript SDK for building Paw messenger agents.
 *
 * ```ts
 * const agent = new PawAgent('my-agent-token');
 *
 * agent.onMessage(async (ctx, streaming) => {
 *   await streaming.send(`Echo: ${ctx.message.content}`);
 * });
 *
 * await agent.connect();
 * ```
 */
export class PawAgent {
  private readonly token: string;
  private readonly serverUrl: string;
  private handler: MessageHandler | null = null;
  private ws: WebSocket | null = null;

  constructor(token: string, options?: AgentOptions) {
    this.token = token;
    this.serverUrl = (options?.serverUrl ?? DEFAULT_SERVER_URL).replace(
      /\/$/,
      '',
    );
  }

  /**
   * Register the message handler.
   *
   * The handler receives the conversation context and a streaming helper.
   * Return a string for a simple response, or use `streaming.*` for
   * streaming / tool frames.
   */
  onMessage(handler: MessageHandler): void {
    this.handler = handler;
  }

  /** Open the agent WebSocket and begin processing incoming contexts. */
  async connect(): Promise<void> {
    const url = `${this.serverUrl}/agent/ws?token=${this.token}`;
    const ws = new WebSocket(url);
    this.ws = ws;

    return new Promise<void>((resolve, reject) => {
      ws.once('open', () => resolve());
      ws.once('error', (err) => reject(err));

      ws.on('message', (raw: WebSocket.RawData) => {
        void this.handleRaw(ws, raw);
      });
    });
  }

  /** Gracefully close the WebSocket connection. */
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  // ─── Internal ────────────────────────────────────────────────────────

  /** @internal Parse and dispatch a raw WS message. */
  private async handleRaw(
    ws: WebSocket,
    raw: WebSocket.RawData,
  ): Promise<void> {
    if (!this.handler) return;

    const data: unknown = JSON.parse(raw.toString());
    const ctx = PawAgent.parseContext(data);
    if (!ctx) return;

    const streaming = new StreamingContextImpl(ws, ctx);
    const result = await this.handler(ctx, streaming);

    if (typeof result === 'string') {
      await streaming.send(result);
    }
  }

  /**
   * Parse a raw JSON payload into a ConversationContext.
   * Returns `null` if the payload is invalid.
   *
   * @internal Exposed as static for unit-testing without a live WS.
   */
  static parseContext(data: unknown): ConversationContext | null {
    if (typeof data !== 'object' || data === null) return null;

    const obj = data as Record<string, unknown>;

    try {
      const message = PawAgent.parseMessage(obj['message']);
      const v = obj['v'];
      if (typeof v !== 'number') return null;

      const conversationId = obj['conversation_id'];
      if (typeof conversationId !== 'string') return null;

      const recentRaw = obj['recent_messages'];
      if (!Array.isArray(recentRaw)) return null;

      const recentMessages = recentRaw.map((m: unknown) =>
        PawAgent.parseMessage(m),
      );

      return {
        v,
        message,
        conversation_id: conversationId,
        recent_messages: recentMessages,
      };
    } catch {
      return null;
    }
  }

  /** @internal Parse a single message object. */
  static parseMessage(raw: unknown): Message {
    if (typeof raw !== 'object' || raw === null) {
      throw new TypeError('message must be an object');
    }
    const obj = raw as Record<string, unknown>;

    return {
      id: String(obj['id']),
      conversation_id: String(obj['conversation_id']),
      sender_id: String(obj['sender_id']),
      content: String(obj['content']),
      format: (obj['format'] as 'plain' | 'markdown') ?? 'plain',
      seq: Number(obj['seq']),
      created_at: String(obj['created_at']),
    };
  }
}
