/** Protocol version — all outgoing WS messages include `v: 1`. */
export declare const PROTOCOL_VERSION = 1;
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
export interface AgentOptions {
    /** WebSocket server URL. Default: `ws://localhost:3000` */
    serverUrl?: string;
}
export interface StreamChunk {
    delta?: string;
    tool?: string;
    label?: string;
}
export type AgentStreamFrame = {
    type: 'stream_start';
    v: number;
    conversation_id: string;
    agent_id: string;
    stream_id: string;
} | {
    type: 'content_delta';
    v: number;
    stream_id: string;
    delta: string;
} | {
    type: 'tool_start';
    v: number;
    stream_id: string;
    tool: string;
    label: string;
} | {
    type: 'tool_end';
    v: number;
    stream_id: string;
    tool: string;
} | {
    type: 'stream_end';
    v: number;
    stream_id: string;
    tokens: number;
    duration_ms: number;
};
export interface AgentResponse {
    v: number;
    conversation_id: string;
    content: string;
    format: string;
}
export type MessageHandler = (ctx: ConversationContext, streaming: StreamingContext) => Promise<string | void>;
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
