from datetime import datetime, timezone

from paw_agent_sdk.agent import PawAgent
from paw_agent_sdk.models import ConversationContext, Message, StreamChunk


def test_message_dataclass_creation_and_field_access():
    created_at = datetime.now(tz=timezone.utc)
    message = Message(
        id="msg-1",
        conversation_id="conv-1",
        sender_id="user-1",
        content="hello",
        format="markdown",
        seq=1,
        created_at=created_at,
    )

    assert message.id == "msg-1"
    assert message.conversation_id == "conv-1"
    assert message.content == "hello"
    assert message.created_at == created_at


def test_conversation_context_with_recent_messages():
    created_at = datetime.now(tz=timezone.utc)
    msg = Message(
        id="msg-1",
        conversation_id="conv-1",
        sender_id="user-1",
        content="current",
        format="plain",
        seq=2,
        created_at=created_at,
    )
    recent = Message(
        id="msg-0",
        conversation_id="conv-1",
        sender_id="user-2",
        content="recent",
        format="markdown",
        seq=1,
        created_at=created_at,
    )

    ctx = ConversationContext(
        v=1, message=msg, conversation_id="conv-1", recent_messages=[recent]
    )
    assert ctx.v == 1
    assert len(ctx.recent_messages) == 1
    assert ctx.recent_messages[0].id == "msg-0"


def test_stream_chunk_with_optional_tool_fields():
    chunk = StreamChunk(
        stream_id="stream-1", delta="", tool="search", label="Searching"
    )
    assert chunk.stream_id == "stream-1"
    assert chunk.delta == ""
    assert chunk.tool == "search"
    assert chunk.label == "Searching"


def test_parse_context_with_valid_data():
    agent = PawAgent(token="test-token")
    parse_context = getattr(agent, "_parse_context")
    parsed = parse_context(
        {
            "v": 1,
            "conversation_id": "conv-123",
            "message": {
                "id": "msg-123",
                "conversation_id": "conv-123",
                "sender_id": "user-123",
                "content": "hello",
                "format": "markdown",
                "seq": 42,
                "created_at": "2026-01-01T00:00:00Z",
            },
            "recent_messages": [
                {
                    "id": "msg-122",
                    "conversation_id": "conv-123",
                    "sender_id": "user-456",
                    "content": "prev",
                    "format": "plain",
                    "seq": 41,
                    "created_at": "2026-01-01T00:00:00Z",
                }
            ],
        }
    )

    assert parsed is not None
    assert parsed.conversation_id == "conv-123"
    assert parsed.message.id == "msg-123"
    assert parsed.message.seq == 42
    assert parsed.message.created_at == datetime(2026, 1, 1, 0, 0, tzinfo=timezone.utc)
    assert len(parsed.recent_messages) == 1


def test_parse_context_with_missing_fields_returns_none():
    agent = PawAgent(token="test-token")
    parse_context = getattr(agent, "_parse_context")
    parsed = parse_context(
        {
            "v": 1,
            "conversation_id": "conv-123",
            "message": {
                "id": "msg-123",
                "conversation_id": "conv-123",
                "sender_id": "user-123",
            },
        }
    )
    assert parsed is None
