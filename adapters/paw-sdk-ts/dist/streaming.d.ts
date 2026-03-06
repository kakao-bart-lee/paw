import { ConversationContext, StreamingContext as IStreamingContext } from './types';
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
export declare class StreamingContextImpl implements IStreamingContext {
    private readonly ws;
    private readonly ctx;
    private readonly streamId;
    private started;
    constructor(ws: WsSend, ctx: ConversationContext);
    /** Send a complete (non-streaming) text response. */
    send(text: string): Promise<void>;
    /** Stream an async iterable of string tokens as content_delta frames. */
    stream(tokens: AsyncIterable<string>): Promise<void>;
    /** Emit a tool_start frame. */
    tool(name: string, label: string): Promise<void>;
    /** Emit a tool_end frame. */
    toolEnd(name: string): Promise<void>;
    private ensureStarted;
    private end;
}
