import { describe, expect, test } from '@jest/globals';
import { PawAgent } from '../agent';
import { StreamingContextImpl } from '../streaming';
import { PROTOCOL_VERSION } from '../types';

const VALID_CONTEXT = {
  v: 1,
  conversation_id: 'conv-1',
  message: {
    id: 'msg-1',
    conversation_id: 'conv-1',
    sender_id: 'user-1',
    content: 'hello',
    format: 'markdown',
    seq: 7,
    created_at: '2026-01-01T00:00:00Z',
  },
  recent_messages: [],
};

describe('PawAgent.parseContext', () => {
  test('parses valid InboundContext JSON', () => {
    const ctx = PawAgent.parseContext(VALID_CONTEXT);
    expect(ctx).not.toBeNull();
    expect(ctx!.conversation_id).toBe('conv-1');
    expect(ctx!.message.content).toBe('hello');
    expect(ctx!.message.seq).toBe(7);
    expect(ctx!.v).toBe(1);
  });

  test('returns null for missing v field', () => {
    const { v, ...noVersion } = VALID_CONTEXT;
    expect(PawAgent.parseContext(noVersion)).toBeNull();
  });

  test('returns null for missing conversation_id', () => {
    const { conversation_id, ...noConvId } = VALID_CONTEXT;
    expect(PawAgent.parseContext(noConvId)).toBeNull();
  });

  test('returns null for non-object input', () => {
    expect(PawAgent.parseContext(null)).toBeNull();
    expect(PawAgent.parseContext('string')).toBeNull();
    expect(PawAgent.parseContext(42)).toBeNull();
  });

  test('parses context with recent_messages', () => {
    const withRecent = {
      ...VALID_CONTEXT,
      recent_messages: [
        {
          id: 'msg-0',
          conversation_id: 'conv-1',
          sender_id: 'user-1',
          content: 'earlier',
          format: 'plain',
          seq: 6,
          created_at: '2025-12-31T23:59:00Z',
        },
      ],
    };
    const ctx = PawAgent.parseContext(withRecent);
    expect(ctx).not.toBeNull();
    expect(ctx!.recent_messages).toHaveLength(1);
    expect(ctx!.recent_messages[0].content).toBe('earlier');
  });
});

describe('PawAgent.parseMessage', () => {
  test('parses valid message object', () => {
    const msg = PawAgent.parseMessage(VALID_CONTEXT.message);
    expect(msg.id).toBe('msg-1');
    expect(msg.content).toBe('hello');
    expect(msg.format).toBe('markdown');
    expect(msg.seq).toBe(7);
  });

  test('throws for non-object input', () => {
    expect(() => PawAgent.parseMessage(null)).toThrow('message must be an object');
    expect(() => PawAgent.parseMessage('str')).toThrow('message must be an object');
  });
});

describe('StreamingContext', () => {
  test('send() emits response with v=1', () => {
    const sent: string[] = [];
    const mockWs = { send: (data: string) => sent.push(data) };

    const ctx = PawAgent.parseContext(VALID_CONTEXT)!;
    const streaming = new StreamingContextImpl(mockWs, ctx);

    streaming.send('pong');
    const parsed = JSON.parse(sent[0]);
    expect(parsed.v).toBe(PROTOCOL_VERSION);
    expect(parsed.content).toBe('pong');
    expect(parsed.format).toBe('markdown');
    expect(parsed.conversation_id).toBe('conv-1');
  });

  test('stream() emits stream_start, content_delta(s), stream_end with v=1', async () => {
    const sent: string[] = [];
    const mockWs = { send: (data: string) => sent.push(data) };

    const ctx = PawAgent.parseContext(VALID_CONTEXT)!;
    const streaming = new StreamingContextImpl(mockWs, ctx);

    async function* tokens() {
      yield 'Hello';
      yield ' World';
    }

    await streaming.stream(tokens());

    expect(sent.length).toBe(4); // stream_start + 2 deltas + stream_end

    const start = JSON.parse(sent[0]);
    expect(start.type).toBe('stream_start');
    expect(start.v).toBe(1);
    expect(start.conversation_id).toBe('conv-1');

    const delta1 = JSON.parse(sent[1]);
    expect(delta1.type).toBe('content_delta');
    expect(delta1.v).toBe(1);
    expect(delta1.delta).toBe('Hello');

    const delta2 = JSON.parse(sent[2]);
    expect(delta2.delta).toBe(' World');

    const end = JSON.parse(sent[3]);
    expect(end.type).toBe('stream_end');
    expect(end.v).toBe(1);
  });

  test('tool() and toolEnd() emit frames with v=1', async () => {
    const sent: string[] = [];
    const mockWs = { send: (data: string) => sent.push(data) };

    const ctx = PawAgent.parseContext(VALID_CONTEXT)!;
    const streaming = new StreamingContextImpl(mockWs, ctx);

    await streaming.tool('search', 'Searching...');
    await streaming.toolEnd('search');

    // stream_start (auto) + tool_start + tool_end = 3 frames
    expect(sent.length).toBe(3);

    const toolStart = JSON.parse(sent[1]);
    expect(toolStart.type).toBe('tool_start');
    expect(toolStart.v).toBe(1);
    expect(toolStart.tool).toBe('search');
    expect(toolStart.label).toBe('Searching...');

    const toolEnd = JSON.parse(sent[2]);
    expect(toolEnd.type).toBe('tool_end');
    expect(toolEnd.v).toBe(1);
    expect(toolEnd.tool).toBe('search');
  });

  test('all outgoing frames include v: 1', async () => {
    const sent: string[] = [];
    const mockWs = { send: (data: string) => sent.push(data) };

    const ctx = PawAgent.parseContext(VALID_CONTEXT)!;
    const streaming = new StreamingContextImpl(mockWs, ctx);

    async function* oneToken() {
      yield 'tok';
    }
    await streaming.stream(oneToken());

    for (const raw of sent) {
      const parsed = JSON.parse(raw);
      expect(parsed.v).toBe(1);
    }
  });
});
