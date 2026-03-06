import { AgentOptions, ConversationContext, Message, MessageHandler } from './types';
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
export declare class PawAgent {
    private readonly token;
    private readonly serverUrl;
    private handler;
    private ws;
    constructor(token: string, options?: AgentOptions);
    /**
     * Register the message handler.
     *
     * The handler receives the conversation context and a streaming helper.
     * Return a string for a simple response, or use `streaming.*` for
     * streaming / tool frames.
     */
    onMessage(handler: MessageHandler): void;
    /** Open the agent WebSocket and begin processing incoming contexts. */
    connect(): Promise<void>;
    /** Gracefully close the WebSocket connection. */
    disconnect(): void;
    /** @internal Parse and dispatch a raw WS message. */
    private handleRaw;
    /**
     * Parse a raw JSON payload into a ConversationContext.
     * Returns `null` if the payload is invalid.
     *
     * @internal Exposed as static for unit-testing without a live WS.
     */
    static parseContext(data: unknown): ConversationContext | null;
    /** @internal Parse a single message object. */
    static parseMessage(raw: unknown): Message;
}
