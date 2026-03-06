"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.StreamingContextImpl = void 0;
const node_crypto_1 = require("node:crypto");
const types_1 = require("./types");
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
class StreamingContextImpl {
    constructor(ws, ctx) {
        this.ws = ws;
        this.ctx = ctx;
        this.started = false;
        this.streamId = (0, node_crypto_1.randomUUID)();
    }
    /** Send a complete (non-streaming) text response. */
    async send(text) {
        const msg = {
            v: types_1.PROTOCOL_VERSION,
            conversation_id: this.ctx.conversation_id,
            content: text,
            format: 'markdown',
        };
        this.ws.send(JSON.stringify(msg));
    }
    /** Stream an async iterable of string tokens as content_delta frames. */
    async stream(tokens) {
        await this.ensureStarted();
        for await (const token of tokens) {
            const frame = {
                type: 'content_delta',
                v: types_1.PROTOCOL_VERSION,
                stream_id: this.streamId,
                delta: token,
            };
            this.ws.send(JSON.stringify(frame));
        }
        await this.end();
    }
    /** Emit a tool_start frame. */
    async tool(name, label) {
        await this.ensureStarted();
        const frame = {
            type: 'tool_start',
            v: types_1.PROTOCOL_VERSION,
            stream_id: this.streamId,
            tool: name,
            label,
        };
        this.ws.send(JSON.stringify(frame));
    }
    /** Emit a tool_end frame. */
    async toolEnd(name) {
        const frame = {
            type: 'tool_end',
            v: types_1.PROTOCOL_VERSION,
            stream_id: this.streamId,
            tool: name,
        };
        this.ws.send(JSON.stringify(frame));
    }
    // ─── Internal ────────────────────────────────────────────────────────
    async ensureStarted() {
        if (this.started)
            return;
        this.started = true;
        const frame = {
            type: 'stream_start',
            v: types_1.PROTOCOL_VERSION,
            conversation_id: this.ctx.conversation_id,
            agent_id: '00000000-0000-0000-0000-000000000000',
            stream_id: this.streamId,
        };
        this.ws.send(JSON.stringify(frame));
    }
    async end() {
        const frame = {
            type: 'stream_end',
            v: types_1.PROTOCOL_VERSION,
            stream_id: this.streamId,
            tokens: 0,
            duration_ms: 0,
        };
        this.ws.send(JSON.stringify(frame));
    }
}
exports.StreamingContextImpl = StreamingContextImpl;
