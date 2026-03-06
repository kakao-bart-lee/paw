// GENERATED FROM: paw-proto/src/lib.rs
// Keep in sync with Rust types manually.
//
// All messages include v: 1 (protocol version field).

import 'package:json_annotation/json_annotation.dart';
import 'package:uuid/uuid.dart';

part 'messages.g.dart';

const int kProtocolVersion = 1;

// ─── Client Messages ───────────────────────────────────────────────────────

@JsonSerializable()
class ConnectMsg {
  final int v;
  final String type;
  final String token;

  const ConnectMsg({
    this.v = kProtocolVersion,
    this.type = 'connect',
    required this.token,
  });

  Map<String, dynamic> toJson() => _$ConnectMsgToJson(this);
}

@JsonSerializable()
class MessageSendMsg {
  final int v;
  final String type;
  @JsonKey(name: 'conversation_id')
  final String conversationId;
  final String content;
  final String format;
  final List<Map<String, dynamic>> blocks;
  @JsonKey(name: 'idempotency_key')
  final String idempotencyKey;

  MessageSendMsg({
    this.v = kProtocolVersion,
    this.type = 'message_send',
    required this.conversationId,
    required this.content,
    this.format = 'markdown',
    this.blocks = const [],
    String? idempotencyKey,
  }) : idempotencyKey = idempotencyKey ?? const Uuid().v4();

  Map<String, dynamic> toJson() => _$MessageSendMsgToJson(this);
}

@JsonSerializable()
class TypingMsg {
  final int v;
  final String type; // 'typing_start' or 'typing_stop'
  @JsonKey(name: 'conversation_id')
  final String conversationId;

  const TypingMsg({
    this.v = kProtocolVersion,
    required this.type,
    required this.conversationId,
  });

  Map<String, dynamic> toJson() => _$TypingMsgToJson(this);
}

@JsonSerializable()
class MessageAckMsg {
  final int v;
  final String type;
  @JsonKey(name: 'conversation_id')
  final String conversationId;
  @JsonKey(name: 'last_seq')
  final int lastSeq;

  const MessageAckMsg({
    this.v = kProtocolVersion,
    this.type = 'message_ack',
    required this.conversationId,
    required this.lastSeq,
  });

  Map<String, dynamic> toJson() => _$MessageAckMsgToJson(this);
}

@JsonSerializable()
class SyncMsg {
  final int v;
  final String type;
  @JsonKey(name: 'conversation_id')
  final String conversationId;
  @JsonKey(name: 'last_seq')
  final int lastSeq;

  const SyncMsg({
    this.v = kProtocolVersion,
    this.type = 'sync',
    required this.conversationId,
    required this.lastSeq,
  });

  Map<String, dynamic> toJson() => _$SyncMsgToJson(this);
}

// ─── Server Messages ────────────────────────────────────────────────────────

/// Parse any server message from JSON
ServerMessage parseServerMessage(Map<String, dynamic> json) {
  final type = json['type'] as String;
  final v = json['v'] as int?;

  if (v == null) {
    throw const FormatException('Missing required v field in server message');
  }

  return switch (type) {
    'hello_ok' => HelloOkMsg.fromJson(json),
    'hello_error' => HelloErrorMsg.fromJson(json),
    'message_received' => MessageReceivedMsg.fromJson(json),
    'typing_start' => ServerTypingMsg.fromJson(json),
    'typing_stop' => ServerTypingMsg.fromJson(json),
    'presence_update' => PresenceUpdateMsg.fromJson(json),
    'stream_start' => StreamStartMsg.fromJson(json),
    'content_delta' => ContentDeltaMsg.fromJson(json),
    'tool_start' => ToolStartMsg.fromJson(json),
    'tool_end' => ToolEndMsg.fromJson(json),
    'stream_end' => StreamEndMsg.fromJson(json),
    _ => UnknownMsg(type: type, json: json),
  };
}

sealed class ServerMessage {
  int get v;
  String get type;
}

@JsonSerializable()
class HelloOkMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  @JsonKey(name: 'user_id')
  final String userId;
  @JsonKey(name: 'server_time')
  final DateTime serverTime;

  const HelloOkMsg({
    required this.v,
    this.type = 'hello_ok',
    required this.userId,
    required this.serverTime,
  });

  factory HelloOkMsg.fromJson(Map<String, dynamic> json) =>
      _$HelloOkMsgFromJson(json);
}

@JsonSerializable()
class HelloErrorMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  final String code;
  final String message;

  const HelloErrorMsg({
    required this.v,
    this.type = 'hello_error',
    required this.code,
    required this.message,
  });

  factory HelloErrorMsg.fromJson(Map<String, dynamic> json) =>
      _$HelloErrorMsgFromJson(json);
}

@JsonSerializable()
class MessageReceivedMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  final String id;
  @JsonKey(name: 'conversation_id')
  final String conversationId;
  @JsonKey(name: 'sender_id')
  final String senderId;
  final String content;
  final String format;
  final int seq;
  @JsonKey(name: 'created_at')
  final DateTime createdAt;
  final List<Map<String, dynamic>> blocks;

  const MessageReceivedMsg({
    required this.v,
    this.type = 'message_received',
    required this.id,
    required this.conversationId,
    required this.senderId,
    required this.content,
    this.format = 'markdown',
    required this.seq,
    required this.createdAt,
    this.blocks = const [],
  });

  factory MessageReceivedMsg.fromJson(Map<String, dynamic> json) =>
      _$MessageReceivedMsgFromJson(json);
}

@JsonSerializable()
class ServerTypingMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  @JsonKey(name: 'conversation_id')
  final String conversationId;
  @JsonKey(name: 'user_id')
  final String userId;

  const ServerTypingMsg({
    required this.v,
    required this.type,
    required this.conversationId,
    required this.userId,
  });

  factory ServerTypingMsg.fromJson(Map<String, dynamic> json) =>
      _$ServerTypingMsgFromJson(json);
}

@JsonSerializable()
class PresenceUpdateMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  @JsonKey(name: 'user_id')
  final String userId;
  final bool online;

  const PresenceUpdateMsg({
    required this.v,
    this.type = 'presence_update',
    required this.userId,
    required this.online,
  });

  factory PresenceUpdateMsg.fromJson(Map<String, dynamic> json) =>
      _$PresenceUpdateMsgFromJson(json);
}

// ─── Phase 2 Streaming (Reserved) ──────────────────────────────────────────

@JsonSerializable()
class StreamStartMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  @JsonKey(name: 'conversation_id')
  final String conversationId;
  @JsonKey(name: 'agent_id')
  final String agentId;
  @JsonKey(name: 'stream_id')
  final String streamId;

  const StreamStartMsg({
    required this.v,
    this.type = 'stream_start',
    required this.conversationId,
    required this.agentId,
    required this.streamId,
  });

  factory StreamStartMsg.fromJson(Map<String, dynamic> json) =>
      _$StreamStartMsgFromJson(json);
}

@JsonSerializable()
class ContentDeltaMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  @JsonKey(name: 'stream_id')
  final String streamId;
  final String delta;

  const ContentDeltaMsg({
    required this.v,
    this.type = 'content_delta',
    required this.streamId,
    required this.delta,
  });

  factory ContentDeltaMsg.fromJson(Map<String, dynamic> json) =>
      _$ContentDeltaMsgFromJson(json);
}

@JsonSerializable()
class ToolStartMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  @JsonKey(name: 'stream_id')
  final String streamId;
  final String tool;
  final String label;

  const ToolStartMsg({
    required this.v,
    this.type = 'tool_start',
    required this.streamId,
    required this.tool,
    required this.label,
  });

  factory ToolStartMsg.fromJson(Map<String, dynamic> json) =>
      _$ToolStartMsgFromJson(json);
}

@JsonSerializable()
class ToolEndMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  @JsonKey(name: 'stream_id')
  final String streamId;
  final String tool;

  const ToolEndMsg({
    required this.v,
    this.type = 'tool_end',
    required this.streamId,
    required this.tool,
  });

  factory ToolEndMsg.fromJson(Map<String, dynamic> json) =>
      _$ToolEndMsgFromJson(json);
}

@JsonSerializable()
class StreamEndMsg implements ServerMessage {
  @override
  final int v;
  @override
  final String type;
  @JsonKey(name: 'stream_id')
  final String streamId;
  final int tokens;
  @JsonKey(name: 'duration_ms')
  final int durationMs;

  const StreamEndMsg({
    required this.v,
    this.type = 'stream_end',
    required this.streamId,
    required this.tokens,
    required this.durationMs,
  });

  factory StreamEndMsg.fromJson(Map<String, dynamic> json) =>
      _$StreamEndMsgFromJson(json);
}

class UnknownMsg implements ServerMessage {
  @override
  final int v = kProtocolVersion;
  @override
  final String type;
  final Map<String, dynamic> json;

  const UnknownMsg({required this.type, required this.json});
}
