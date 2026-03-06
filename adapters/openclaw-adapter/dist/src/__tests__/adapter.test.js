"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const globals_1 = require("@jest/globals");
const adapter_1 = require("../adapter");
const e2ee_1 = require("../e2ee");
(0, globals_1.describe)('OpenClaw adapter protocol helpers', () => {
    (0, globals_1.test)('parses valid InboundContext JSON', () => {
        const raw = JSON.stringify({
            v: 1,
            conversation_id: 'conv-1',
            message: {
                type: 'message_received',
                v: 1,
                id: 'msg-1',
                conversation_id: 'conv-1',
                sender_id: 'user-1',
                content: 'hello',
                format: 'markdown',
                seq: 7,
                created_at: '2026-01-01T00:00:00Z',
            },
            recent_messages: [],
        });
        const parsed = (0, adapter_1.parseInboundContext)(raw);
        (0, globals_1.expect)(parsed).not.toBeNull();
        (0, globals_1.expect)(parsed?.conversation_id).toBe('conv-1');
        (0, globals_1.expect)(parsed?.message.content).toBe('hello');
    });
    (0, globals_1.test)('returns null for invalid InboundContext JSON', () => {
        const raw = '{"v":1,"conversation_id":"conv-1"';
        const parsed = (0, adapter_1.parseInboundContext)(raw);
        (0, globals_1.expect)(parsed).toBeNull();
    });
    (0, globals_1.test)('serializes AgentResponse with required v=1', () => {
        const response = (0, adapter_1.serializeAgentResponse)('conv-1', 'pong');
        (0, globals_1.expect)(response.v).toBe(1);
        (0, globals_1.expect)(response.format).toBe('markdown');
        (0, globals_1.expect)(response.content).toBe('pong');
    });
    (0, globals_1.test)('AgentStreamFrame discriminates by type', () => {
        const frame = {
            type: 'content_delta',
            v: 1,
            stream_id: 'stream-1',
            delta: 'token',
        };
        if (frame.type === 'content_delta') {
            (0, globals_1.expect)(frame.delta).toBe('token');
        }
        else {
            throw new Error('expected content_delta frame');
        }
    });
    (0, globals_1.test)('looksLikeCiphertext returns true for long base64 string', () => {
        const nonceAndPayload = 'A'.repeat(64);
        (0, globals_1.expect)((0, e2ee_1.looksLikeCiphertext)(nonceAndPayload)).toBe(true);
    });
    (0, globals_1.test)('looksLikeCiphertext returns false for short string', () => {
        (0, globals_1.expect)((0, e2ee_1.looksLikeCiphertext)('aGVsbG8=')).toBe(false);
    });
    (0, globals_1.test)('E2eeConfig has publicKey and privateKey fields', () => {
        const config = {
            publicKey: 'cHVibGljLWtleQ==',
            privateKey: 'cHJpdmF0ZS1rZXk=',
        };
        (0, globals_1.expect)(config.publicKey).toBe('cHVibGljLWtleQ==');
        (0, globals_1.expect)(config.privateKey).toBe('cHJpdmF0ZS1rZXk=');
    });
});
