# @paw/sdk — TypeScript Agent SDK Quickstart

Build Paw messenger agents in TypeScript with event-driven handlers and streaming support.

## Install

```bash
npm install @paw/sdk
```

## Quickstart — Echo Bot

Create a file named `echo-bot.ts`:

```typescript
import { PawAgent } from '@paw/sdk';

// Initialize the agent with your token
const agent = new PawAgent('your-agent-token', {
  serverUrl: 'ws://localhost:3000',
});

// Register the message handler
agent.onMessage(async (ctx, streaming) => {
  // Simple echo handler that sends a complete response
  await streaming.send(`Echo: ${ctx.message.content}`);
});

// Connect to the Paw server
await agent.connect();
```

## Streaming Bot

Use the `streaming.stream()` method to send a token-by-token response.

```typescript
import { PawAgent } from '@paw/sdk';

const agent = new PawAgent('your-agent-token');

agent.onMessage(async (ctx, streaming) => {
  // Async generator for streaming tokens
  async function* generateTokens() {
    const words = ctx.message.content.split(' ');
    for (const word of words) {
      yield word + ' ';
      await new Promise(resolve => setTimeout(resolve, 100)); // Simulate thinking
    }
  }

  // Stream the tokens to the client
  await streaming.stream(generateTokens());
});

await agent.connect();
```

## Tool Frames

You can emit tool start/end frames during streaming to show progress in the UI.

```typescript
import { PawAgent } from '@paw/sdk';

const agent = new PawAgent('your-agent-token');

agent.onMessage(async (ctx, streaming) => {
  // Start a tool frame
  await streaming.tool('search', 'Searching the web...');
  
  // ... do work ...
  await new Promise(resolve => setTimeout(resolve, 1000));
  
  // End a tool frame
  await streaming.toolEnd('search');

  // Send final content
  await streaming.send('Here are the results.');
});

await agent.connect();
```

## API Reference

### `PawAgent(token, options?)`

- `token`: Your agent's authentication token.
- `options.serverUrl`: WS server URL (default: `ws://localhost:3000`).

### `agent.onMessage(handler)`

Register the message handler. The handler receives `(ctx, streaming)`:

- `ctx.message`: The incoming `Message` object.
- `ctx.conversation_id`: Conversation UUID.
- `ctx.recent_messages`: Recent message history for context.
- `streaming.send(text)`: Send a complete markdown response.
- `streaming.stream(asyncIterable)`: Stream token-by-token.
- `streaming.tool(name, label)`: Emit `tool_start` frame.
- `streaming.toolEnd(name)`: Emit `tool_end` frame.

### `agent.connect()` / `agent.disconnect()`

Open or close the WebSocket connection.

### `Message`

- `id`: Message UUID.
- `content`: Text content.
- `sender_id`: UUID of the sender.
- `format`: "plain" or "markdown".
- `created_at`: ISO 8601 date string.
