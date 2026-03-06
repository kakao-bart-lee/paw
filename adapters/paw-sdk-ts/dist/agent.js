"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.PawAgent = void 0;
const ws_1 = __importDefault(require("ws"));
const streaming_1 = require("./streaming");
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
class PawAgent {
    constructor(token, options) {
        this.handler = null;
        this.ws = null;
        this.token = token;
        this.serverUrl = (options?.serverUrl ?? DEFAULT_SERVER_URL).replace(/\/$/, '');
    }
    /**
     * Register the message handler.
     *
     * The handler receives the conversation context and a streaming helper.
     * Return a string for a simple response, or use `streaming.*` for
     * streaming / tool frames.
     */
    onMessage(handler) {
        this.handler = handler;
    }
    /** Open the agent WebSocket and begin processing incoming contexts. */
    async connect() {
        const url = `${this.serverUrl}/agent/ws?token=${this.token}`;
        const ws = new ws_1.default(url);
        this.ws = ws;
        return new Promise((resolve, reject) => {
            ws.once('open', () => resolve());
            ws.once('error', (err) => reject(err));
            ws.on('message', (raw) => {
                void this.handleRaw(ws, raw);
            });
        });
    }
    /** Gracefully close the WebSocket connection. */
    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }
    // ─── Internal ────────────────────────────────────────────────────────
    /** @internal Parse and dispatch a raw WS message. */
    async handleRaw(ws, raw) {
        if (!this.handler)
            return;
        const data = JSON.parse(raw.toString());
        const ctx = PawAgent.parseContext(data);
        if (!ctx)
            return;
        const streaming = new streaming_1.StreamingContextImpl(ws, ctx);
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
    static parseContext(data) {
        if (typeof data !== 'object' || data === null)
            return null;
        const obj = data;
        try {
            const message = PawAgent.parseMessage(obj['message']);
            const v = obj['v'];
            if (typeof v !== 'number')
                return null;
            const conversationId = obj['conversation_id'];
            if (typeof conversationId !== 'string')
                return null;
            const recentRaw = obj['recent_messages'];
            if (!Array.isArray(recentRaw))
                return null;
            const recentMessages = recentRaw.map((m) => PawAgent.parseMessage(m));
            return {
                v,
                message,
                conversation_id: conversationId,
                recent_messages: recentMessages,
            };
        }
        catch {
            return null;
        }
    }
    /** @internal Parse a single message object. */
    static parseMessage(raw) {
        if (typeof raw !== 'object' || raw === null) {
            throw new TypeError('message must be an object');
        }
        const obj = raw;
        return {
            id: String(obj['id']),
            conversation_id: String(obj['conversation_id']),
            sender_id: String(obj['sender_id']),
            content: String(obj['content']),
            format: obj['format'] ?? 'plain',
            seq: Number(obj['seq']),
            created_at: String(obj['created_at']),
        };
    }
}
exports.PawAgent = PawAgent;
