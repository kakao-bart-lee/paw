import { randomUUID } from 'node:crypto';
import {
  AgentStreamFrame,
  ConversationContext,
  PROTOCOL_VERSION,
  StreamingContext as IStreamingContext,
} from './types';

/** WebSocket-like send interface (for testability). */
export interface WsSend {
  send(data: string): void;
}

/**
 * StreamingContext — provides helpers to send streaming frames over WS.
 *
 * Usage inside a message handler:
 * ```ts
 * agent.onMessage(async (ctx, streaming) => {
 *   await streaming.stream(generateTokens());
 * });
 * ```
 */
export class StreamingContextImpl implements IStreamingContext {
  private readonly streamId: string;
  private started = false;

  constructor(
    private readonly ws: WsSend,
    private readonly ctx: ConversationContext,
  ) {
    this.streamId = randomUUID();
  }

  /** Send a complete (non-streaming) text response. */
  async send(text: string): Promise<void> {
    const msg = {
      v: PROTOCOL_VERSION,
      conversation_id: this.ctx.conversation_id,
      content: text,
      format: 'markdown',
    };
    this.ws.send(JSON.stringify(msg));
  }

  /** Stream an async iterable of string tokens as content_delta frames. */
  async stream(tokens: AsyncIterable<string>): Promise<void> {
    await this.ensureStarted();

    for await (const token of tokens) {
      const frame: AgentStreamFrame = {
        type: 'content_delta',
        v: PROTOCOL_VERSION,
        stream_id: this.streamId,
        delta: token,
      };
      this.ws.send(JSON.stringify(frame));
    }

    await this.end();
  }

  /** Emit a tool_start frame. */
  async tool(name: string, label: string): Promise<void> {
    await this.ensureStarted();

    const frame: AgentStreamFrame = {
      type: 'tool_start',
      v: PROTOCOL_VERSION,
      stream_id: this.streamId,
      tool: name,
      label,
    };
    this.ws.send(JSON.stringify(frame));
  }

  /** Emit a tool_end frame. */
  async toolEnd(name: string): Promise<void> {
    const frame: AgentStreamFrame = {
      type: 'tool_end',
      v: PROTOCOL_VERSION,
      stream_id: this.streamId,
      tool: name,
    };
    this.ws.send(JSON.stringify(frame));
  }

  // ─── Internal ────────────────────────────────────────────────────────

  private async ensureStarted(): Promise<void> {
    if (this.started) return;
    this.started = true;

    const frame: AgentStreamFrame = {
      type: 'stream_start',
      v: PROTOCOL_VERSION,
      conversation_id: this.ctx.conversation_id,
      agent_id: '00000000-0000-0000-0000-000000000000',
      stream_id: this.streamId,
    };
    this.ws.send(JSON.stringify(frame));
  }

  private async end(): Promise<void> {
    const frame: AgentStreamFrame = {
      type: 'stream_end',
      v: PROTOCOL_VERSION,
      stream_id: this.streamId,
      tokens: 0,
      duration_ms: 0,
    };
    this.ws.send(JSON.stringify(frame));
  }
}
