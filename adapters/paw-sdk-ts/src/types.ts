/** Protocol version — all outgoing WS messages include `v: 1`. */
export const PROTOCOL_VERSION = 1;

// ─── Core Types ──────────────────────────────────────────────────────────

export interface Message {
  id: string;
  conversation_id: string;
  sender_id: string;
  content: string;
  format: 'plain' | 'markdown';
  seq: number;
  created_at: string;
}

export interface ConversationContext {
  v: number;
  message: Message;
  conversation_id: string;
  recent_messages: Message[];
}

// ─── Agent Options ───────────────────────────────────────────────────────

export interface AgentOptions {
  /** WebSocket server URL. Default: `ws://localhost:3000` */
  serverUrl?: string;
}

// ─── Stream Chunks ───────────────────────────────────────────────────────

export interface StreamChunk {
  delta?: string;
  tool?: string;
  label?: string;
}

// ─── Agent Stream Frames (outgoing WS) ──────────────────────────────────

export type AgentStreamFrame =
  | {
      type: 'stream_start';
      v: number;
      conversation_id: string;
      agent_id: string;
      stream_id: string;
    }
  | {
      type: 'content_delta';
      v: number;
      stream_id: string;
      delta: string;
    }
  | {
      type: 'tool_start';
      v: number;
      stream_id: string;
      tool: string;
      label: string;
    }
  | {
      type: 'tool_end';
      v: number;
      stream_id: string;
      tool: string;
    }
  | {
      type: 'stream_end';
      v: number;
      stream_id: string;
      tokens: number;
      duration_ms: number;
    };

// ─── Agent Response (outgoing WS — non-streaming) ───────────────────────

export interface AgentResponse {
  v: number;
  conversation_id: string;
  content: string;
  format: string;
}

// ─── Handler Signature ──────────────────────────────────────────────────

export type MessageHandler = (
  ctx: ConversationContext,
  streaming: StreamingContext,
) => Promise<string | void>;

// Forward reference — implemented in streaming.ts
export interface StreamingContext {
  /** Send a complete text response. */
  send(text: string): Promise<void>;
  /** Stream an async iterable of string tokens. */
  stream(tokens: AsyncIterable<string>): Promise<void>;
  /** Emit a tool_start frame (call again with same tool name to emit tool_end). */
  tool(name: string, label: string): Promise<void>;
  /** Emit a tool_end frame. */
  toolEnd(name: string): Promise<void>;
}
