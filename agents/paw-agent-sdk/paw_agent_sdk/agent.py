import json
import uuid
import importlib
from collections.abc import Awaitable, Callable
from datetime import datetime
from typing import Protocol

from .models import ConversationContext, Message, StreamChunk
from .streaming import StreamingResponse


class AgentWebSocket(Protocol):
    async def send(self, data: str) -> None: ...


MessageHandler = Callable[
    [ConversationContext], Awaitable[str | StreamingResponse | None]
]


class PawAgent:
    def __init__(self, token: str, server_url: str = "ws://localhost:38173"):
        self.token: str = token
        self.server_url: str = server_url.rstrip("/")
        self._handler: MessageHandler | None = None

    def on_message(self, func: MessageHandler) -> MessageHandler:
        """Decorator: register async message handler."""
        self._handler = func
        return func

    async def run(self):
        """Connect to Paw server and process incoming contexts."""
        websockets = importlib.import_module("websockets")
        url = f"{self.server_url}/agent/ws?token={self.token}"
        async with websockets.connect(url) as ws:
            async for raw in ws:
                data = json.loads(raw)
                ctx = self._parse_context(data)
                if not ctx or not self._handler:
                    continue

                response = await self._handler(ctx)
                if isinstance(response, StreamingResponse):
                    await self._send_stream(ws, ctx, response)
                elif isinstance(response, str):
                    await self._send_response(ws, ctx.conversation_id, response)

    async def _send_response(
        self, ws: AgentWebSocket, conversation_id: str, content: str
    ) -> None:
        msg = {
            "v": 1,
            "conversation_id": conversation_id,
            "content": content,
            "format": "markdown",
        }
        await ws.send(json.dumps(msg))

    async def _send_stream(
        self, ws: AgentWebSocket, ctx: ConversationContext, streaming: StreamingResponse
    ) -> None:
        stream_id = str(uuid.uuid4())
        await ws.send(
            json.dumps(
                {
                    "type": "stream_start",
                    "v": 1,
                    "conversation_id": ctx.conversation_id,
                    "agent_id": "00000000-0000-0000-0000-000000000000",
                    "stream_id": stream_id,
                }
            )
        )

        async for chunk in streaming:
            if isinstance(chunk, StreamChunk):
                if chunk.tool and chunk.label:
                    await ws.send(
                        json.dumps(
                            {
                                "type": "tool_start",
                                "v": 1,
                                "stream_id": stream_id,
                                "tool": chunk.tool,
                                "label": chunk.label,
                            }
                        )
                    )
                if chunk.delta:
                    await ws.send(
                        json.dumps(
                            {
                                "type": "content_delta",
                                "v": 1,
                                "stream_id": stream_id,
                                "delta": chunk.delta,
                            }
                        )
                    )
                if chunk.tool and not chunk.label:
                    await ws.send(
                        json.dumps(
                            {
                                "type": "tool_end",
                                "v": 1,
                                "stream_id": stream_id,
                                "tool": chunk.tool,
                            }
                        )
                    )
            else:
                await ws.send(
                    json.dumps(
                        {
                            "type": "content_delta",
                            "v": 1,
                            "stream_id": stream_id,
                            "delta": str(chunk),
                        }
                    )
                )

        await ws.send(
            json.dumps(
                {
                    "type": "stream_end",
                    "v": 1,
                    "stream_id": stream_id,
                    "tokens": 0,
                    "duration_ms": 0,
                }
            )
        )

    def _parse_context(self, data: dict[str, object]) -> ConversationContext | None:
        try:
            message = self._parse_message(data["message"])
            recent_raw = data.get("recent_messages", [])
            if not isinstance(recent_raw, list):
                raise TypeError("recent_messages must be a list")
            recent_messages = [self._parse_message(item) for item in recent_raw]
            version = data["v"]
            if not isinstance(version, int):
                raise TypeError("v must be an int")

            return ConversationContext(
                v=version,
                message=message,
                conversation_id=str(data["conversation_id"]),
                recent_messages=recent_messages,
            )
        except (KeyError, TypeError, ValueError):
            return None

    def _parse_message(self, msg_data: object) -> Message:
        if not isinstance(msg_data, dict):
            raise TypeError("message must be an object")

        return Message(
            id=str(msg_data["id"]),
            conversation_id=str(msg_data["conversation_id"]),
            sender_id=str(msg_data["sender_id"]),
            content=str(msg_data["content"]),
            format=str(msg_data.get("format", "plain")),
            seq=int(msg_data["seq"]),
            created_at=self._parse_datetime(msg_data["created_at"]),
        )

    def _parse_datetime(self, raw: object) -> datetime:
        if isinstance(raw, datetime):
            return raw
        if isinstance(raw, str):
            normalized = raw.replace("Z", "+00:00")
            return datetime.fromisoformat(normalized)
        raise ValueError("created_at must be datetime or ISO 8601 string")
