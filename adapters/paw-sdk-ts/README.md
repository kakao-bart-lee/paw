# @paw/sdk — TypeScript Agent SDK

Build Paw messenger agents in TypeScript with event-driven handlers and streaming support.

## Install

```bash
npm install @paw/sdk
```

## Quickstart — Echo Bot

```ts
import { PawAgent } from '@paw/sdk';

const agent = new PawAgent('your-agent-token', {
  serverUrl: 'ws://localhost:3000',
});

agent.onMessage(async (ctx, streaming) => {
  await streaming.send(`Echo: ${ctx.message.content}`);
});

await agent.connect();
```

## Streaming Bot

```ts
import { PawAgent } from '@paw/sdk';

const agent = new PawAgent('your-agent-token');

agent.onMessage(async (ctx, streaming) => {
  async function* generateTokens() {
    for (const word of ctx.message.content.split(' ')) {
      yield word + ' ';
    }
  }

  await streaming.stream(generateTokens());
});

await agent.connect();
```

## Tool Frames

```ts
agent.onMessage(async (ctx, streaming) => {
  await streaming.tool('search', 'Searching the web...');
  // ... do work ...
  await streaming.toolEnd('search');

  await streaming.send('Here are the results.');
});
```

## API

### `PawAgent(token, options?)`

| Param | Type | Description |
|-------|------|-------------|
| `token` | `string` | Agent authentication token |
| `options.serverUrl` | `string` | WS server URL (default: `ws://localhost:3000`) |

### `agent.onMessage(handler)`

Register the message handler. Handler receives `(ctx, streaming)`:

- `ctx.message` — the incoming `Message`
- `ctx.conversation_id` — conversation UUID
- `ctx.recent_messages` — recent message history
- `streaming.send(text)` — send a complete response
- `streaming.stream(asyncIterable)` — stream token-by-token
- `streaming.tool(name, label)` — emit tool_start frame
- `streaming.toolEnd(name)` — emit tool_end frame

### `agent.connect()` / `agent.disconnect()`

Open or close the WebSocket connection.
