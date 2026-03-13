# Paw Agent SDK — Python Quickstart

Build Paw messenger agents in Python with simple decorators and streaming support.

## Install

```bash
pip install paw-agent-sdk
```

## Quickstart — Echo Bot

Create a file named `echo_bot.py`:

```python
import asyncio
from paw_agent_sdk import ConversationContext, PawAgent

# Initialize the agent with your token
agent = PawAgent(token="your-agent-token")

@agent.on_message
async def handle(ctx: ConversationContext) -> str:
    """Simple echo handler that returns a string."""
    return f"Echo: {ctx.message.content}"

if __name__ == "__main__":
    asyncio.run(agent.run())
```

## Streaming Bot

Use the `@stream` decorator to return an async generator for token-by-token streaming.

```python
import asyncio
from paw_agent_sdk import ConversationContext, PawAgent, stream

agent = PawAgent(token="your-agent-token")

@agent.on_message
@stream
async def handle_streaming(ctx: ConversationContext):
    """Streaming handler that yields tokens."""
    words = ctx.message.content.split()
    for word in words:
        yield word + " "
        await asyncio.sleep(0.1)  # Simulate thinking

if __name__ == "__main__":
    asyncio.run(agent.run())
```

## Tool Frames

You can emit tool start/end frames during streaming to show progress in the UI.

```python
import asyncio
from paw_agent_sdk import ConversationContext, PawAgent, StreamChunk, stream

agent = PawAgent(token="your-agent-token")

@agent.on_message
@stream
async def handle_tools(ctx: ConversationContext):
    # Start a tool
    yield StreamChunk(stream_id="", delta="", tool="web_search", label="Searching...")
    await asyncio.sleep(1)
    
    # End a tool
    yield StreamChunk(stream_id="", delta="", tool="web_search")
    
    # Send final content
    yield "Here is what I found."

if __name__ == "__main__":
    asyncio.run(agent.run())
```

## API Reference

### `PawAgent(token, server_url="ws://localhost:38173")`

- `token`: Your agent's authentication token.
- `server_url`: The Paw server WebSocket URL.

### `@agent.on_message`

Decorator to register an async message handler. The handler receives a `ConversationContext` and can return:
- `str`: A complete markdown response.
- `StreamingResponse`: (via `@stream` decorator) An async generator of tokens or `StreamChunk`s.
- `None`: No response.

### `ConversationContext`

- `v`: Protocol version.
- `message`: The incoming `Message` object.
- `conversation_id`: UUID of the conversation.
- `recent_messages`: List of recent `Message` objects for context.

### `Message`

- `id`: Message UUID.
- `content`: Text content.
- `sender_id`: UUID of the sender.
- `format`: "plain" or "markdown".
- `created_at`: Python `datetime` object.
