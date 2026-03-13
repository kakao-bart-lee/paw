"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.OpenClawAdapter = void 0;
exports.parseInboundContext = parseInboundContext;
exports.serializeAgentResponse = serializeAgentResponse;
const ws_1 = __importDefault(require("ws"));
const node_crypto_1 = require("node:crypto");
const e2ee_1 = require("./e2ee");
const PROTOCOL_VERSION = 1;
function parseInboundContext(raw) {
    try {
        const data = JSON.parse(raw);
        if (!isInboundContext(data)) {
            return null;
        }
        return data;
    }
    catch {
        return null;
    }
}
function serializeAgentResponse(conversationId, content, format = 'markdown') {
    return {
        v: PROTOCOL_VERSION,
        conversation_id: conversationId,
        content,
        format,
    };
}
class OpenClawAdapter {
    token;
    ws = null;
    handlers = new Map();
    serverUrl;
    channelConfig;
    agentId;
    defaultFormat;
    e2eeConfig;
    constructor(token, options = {}) {
        this.token = token;
        this.serverUrl = (options.serverUrl ?? 'ws://localhost:38173').replace(/\/$/, '');
        this.channelConfig = options.channelConfig ?? {};
        this.agentId = options.agentId ?? '00000000-0000-0000-0000-000000000000';
        this.defaultFormat = options.defaultFormat ?? 'markdown';
        this.e2eeConfig = options.e2ee ?? null;
    }
    registerChannel(handler) {
        this.handlers.set(handler.name, handler);
        return this;
    }
    async connect() {
        const url = `${this.serverUrl}/agent/ws?token=${this.token}`;
        this.ws = new ws_1.default(url);
        await new Promise((resolve, reject) => {
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
    disconnect() {
        this.ws?.close();
        this.ws = null;
    }
    async routeInboundContext(ctx) {
        let routedContext = ctx;
        const inboundContent = ctx.message.content;
        if (this.e2eeConfig && (0, e2ee_1.looksLikeCiphertext)(inboundContent)) {
            const decrypted = (0, e2ee_1.decryptContent)(this.e2eeConfig.privateKey, inboundContent);
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
                responseContent = (0, e2ee_1.encryptContent)(this.e2eeConfig.publicKey, response);
            }
            const message = serializeAgentResponse(routedContext.conversation_id, responseContent, this.defaultFormat);
            this.ws.send(JSON.stringify(message));
            return;
        }
        const streamId = (0, node_crypto_1.randomUUID)();
        const streamStart = {
            type: 'stream_start',
            v: PROTOCOL_VERSION,
            conversation_id: routedContext.conversation_id,
            agent_id: this.agentId,
            stream_id: streamId,
        };
        this.ws.send(JSON.stringify(streamStart));
        for await (const delta of response) {
            const frame = {
                type: 'content_delta',
                v: PROTOCOL_VERSION,
                stream_id: streamId,
                delta,
            };
            this.ws.send(JSON.stringify(frame));
        }
        const endFrame = {
            type: 'stream_end',
            v: PROTOCOL_VERSION,
            stream_id: streamId,
            tokens: 0,
            duration_ms: 0,
        };
        this.ws.send(JSON.stringify(endFrame));
    }
    resolveHandler(ctx) {
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
exports.OpenClawAdapter = OpenClawAdapter;
function isInboundContext(data) {
    if (!isRecord(data)) {
        return false;
    }
    return (data.v === PROTOCOL_VERSION &&
        typeof data.conversation_id === 'string' &&
        Array.isArray(data.recent_messages) &&
        isMessageReceived(data.message));
}
function isMessageReceived(value) {
    if (!isRecord(value)) {
        return false;
    }
    return (value.v === PROTOCOL_VERSION &&
        value.type === 'message_received' &&
        typeof value.id === 'string' &&
        typeof value.conversation_id === 'string' &&
        typeof value.sender_id === 'string' &&
        typeof value.content === 'string' &&
        (value.format === 'plain' || value.format === 'markdown') &&
        typeof value.seq === 'number' &&
        typeof value.created_at === 'string');
}
function isRecord(value) {
    return typeof value === 'object' && value !== null;
}
