from dataclasses import dataclass
from datetime import datetime


@dataclass
class Message:
    id: str
    conversation_id: str
    sender_id: str
    content: str
    format: str  # "plain" | "markdown"
    seq: int
    created_at: datetime


@dataclass
class ConversationContext:
    v: int
    message: Message
    conversation_id: str
    recent_messages: list[Message]


@dataclass
class StreamChunk:
    stream_id: str
    delta: str  # for content_delta
    tool: str | None = None  # for tool_start/tool_end
    label: str | None = None
