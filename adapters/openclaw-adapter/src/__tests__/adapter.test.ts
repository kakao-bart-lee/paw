import { describe, expect, test } from '@jest/globals';
import {
  parseInboundContext,
  serializeAgentResponse,
} from '../adapter';
import { looksLikeCiphertext } from '../e2ee';
import { AgentStreamFrame, E2eeConfig } from '../types';

describe('OpenClaw adapter protocol helpers', () => {
  test('parses valid InboundContext JSON', () => {
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

    const parsed = parseInboundContext(raw);
    expect(parsed).not.toBeNull();
    expect(parsed?.conversation_id).toBe('conv-1');
    expect(parsed?.message.content).toBe('hello');
  });

  test('returns null for invalid InboundContext JSON', () => {
    const raw = '{"v":1,"conversation_id":"conv-1"';
    const parsed = parseInboundContext(raw);
    expect(parsed).toBeNull();
  });

  test('serializes AgentResponse with required v=1', () => {
    const response = serializeAgentResponse('conv-1', 'pong');
    expect(response.v).toBe(1);
    expect(response.format).toBe('markdown');
    expect(response.content).toBe('pong');
  });

  test('AgentStreamFrame discriminates by type', () => {
    const frame: AgentStreamFrame = {
      type: 'content_delta',
      v: 1,
      stream_id: 'stream-1',
      delta: 'token',
    };

    if (frame.type === 'content_delta') {
      expect(frame.delta).toBe('token');
    } else {
      throw new Error('expected content_delta frame');
    }
  });

  test('looksLikeCiphertext returns true for long base64 string', () => {
    const nonceAndPayload = 'A'.repeat(64);
    expect(looksLikeCiphertext(nonceAndPayload)).toBe(true);
  });

  test('looksLikeCiphertext returns false for short string', () => {
    expect(looksLikeCiphertext('aGVsbG8=')).toBe(false);
  });

  test('E2eeConfig has publicKey and privateKey fields', () => {
    const config: E2eeConfig = {
      publicKey: 'cHVibGljLWtleQ==',
      privateKey: 'cHJpdmF0ZS1rZXk=',
    };

    expect(config.publicKey).toBe('cHVibGljLWtleQ==');
    expect(config.privateKey).toBe('cHJpdmF0ZS1rZXk=');
  });
});
