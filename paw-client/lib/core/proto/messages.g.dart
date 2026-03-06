// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'messages.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ConnectMsg _$ConnectMsgFromJson(Map<String, dynamic> json) => ConnectMsg(
      v: (json['v'] as num?)?.toInt() ?? kProtocolVersion,
      type: json['type'] as String? ?? 'connect',
      token: json['token'] as String,
    );

Map<String, dynamic> _$ConnectMsgToJson(ConnectMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'token': instance.token,
    };

MessageSendMsg _$MessageSendMsgFromJson(Map<String, dynamic> json) =>
    MessageSendMsg(
      v: (json['v'] as num?)?.toInt() ?? kProtocolVersion,
      type: json['type'] as String? ?? 'message_send',
      conversationId: json['conversation_id'] as String,
      content: json['content'] as String,
      format: json['format'] as String? ?? 'markdown',
      blocks: (json['blocks'] as List<dynamic>?)
              ?.map((e) => e as Map<String, dynamic>)
              .toList() ??
          const [],
      idempotencyKey: json['idempotency_key'] as String?,
    );

Map<String, dynamic> _$MessageSendMsgToJson(MessageSendMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'conversation_id': instance.conversationId,
      'content': instance.content,
      'format': instance.format,
      'blocks': instance.blocks,
      'idempotency_key': instance.idempotencyKey,
    };

TypingMsg _$TypingMsgFromJson(Map<String, dynamic> json) => TypingMsg(
      v: (json['v'] as num?)?.toInt() ?? kProtocolVersion,
      type: json['type'] as String,
      conversationId: json['conversation_id'] as String,
    );

Map<String, dynamic> _$TypingMsgToJson(TypingMsg instance) => <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'conversation_id': instance.conversationId,
    };

MessageAckMsg _$MessageAckMsgFromJson(Map<String, dynamic> json) =>
    MessageAckMsg(
      v: (json['v'] as num?)?.toInt() ?? kProtocolVersion,
      type: json['type'] as String? ?? 'message_ack',
      conversationId: json['conversation_id'] as String,
      lastSeq: (json['last_seq'] as num).toInt(),
    );

Map<String, dynamic> _$MessageAckMsgToJson(MessageAckMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'conversation_id': instance.conversationId,
      'last_seq': instance.lastSeq,
    };

SyncMsg _$SyncMsgFromJson(Map<String, dynamic> json) => SyncMsg(
      v: (json['v'] as num?)?.toInt() ?? kProtocolVersion,
      type: json['type'] as String? ?? 'sync',
      conversationId: json['conversation_id'] as String,
      lastSeq: (json['last_seq'] as num).toInt(),
    );

Map<String, dynamic> _$SyncMsgToJson(SyncMsg instance) => <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'conversation_id': instance.conversationId,
      'last_seq': instance.lastSeq,
    };

HelloOkMsg _$HelloOkMsgFromJson(Map<String, dynamic> json) => HelloOkMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'hello_ok',
      userId: json['user_id'] as String,
      serverTime: DateTime.parse(json['server_time'] as String),
    );

Map<String, dynamic> _$HelloOkMsgToJson(HelloOkMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'user_id': instance.userId,
      'server_time': instance.serverTime.toIso8601String(),
    };

HelloErrorMsg _$HelloErrorMsgFromJson(Map<String, dynamic> json) =>
    HelloErrorMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'hello_error',
      code: json['code'] as String,
      message: json['message'] as String,
    );

Map<String, dynamic> _$HelloErrorMsgToJson(HelloErrorMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'code': instance.code,
      'message': instance.message,
    };

MessageReceivedMsg _$MessageReceivedMsgFromJson(Map<String, dynamic> json) =>
    MessageReceivedMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'message_received',
      id: json['id'] as String,
      conversationId: json['conversation_id'] as String,
      senderId: json['sender_id'] as String,
      content: json['content'] as String,
      format: json['format'] as String? ?? 'markdown',
      seq: (json['seq'] as num).toInt(),
      createdAt: DateTime.parse(json['created_at'] as String),
      blocks: (json['blocks'] as List<dynamic>?)
              ?.map((e) => e as Map<String, dynamic>)
              .toList() ??
          const [],
    );

Map<String, dynamic> _$MessageReceivedMsgToJson(MessageReceivedMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'id': instance.id,
      'conversation_id': instance.conversationId,
      'sender_id': instance.senderId,
      'content': instance.content,
      'format': instance.format,
      'seq': instance.seq,
      'created_at': instance.createdAt.toIso8601String(),
      'blocks': instance.blocks,
    };

ServerTypingMsg _$ServerTypingMsgFromJson(Map<String, dynamic> json) =>
    ServerTypingMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String,
      conversationId: json['conversation_id'] as String,
      userId: json['user_id'] as String,
    );

Map<String, dynamic> _$ServerTypingMsgToJson(ServerTypingMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'conversation_id': instance.conversationId,
      'user_id': instance.userId,
    };

PresenceUpdateMsg _$PresenceUpdateMsgFromJson(Map<String, dynamic> json) =>
    PresenceUpdateMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'presence_update',
      userId: json['user_id'] as String,
      online: json['online'] as bool,
    );

Map<String, dynamic> _$PresenceUpdateMsgToJson(PresenceUpdateMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'user_id': instance.userId,
      'online': instance.online,
    };

StreamStartMsg _$StreamStartMsgFromJson(Map<String, dynamic> json) =>
    StreamStartMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'stream_start',
      conversationId: json['conversation_id'] as String,
      agentId: json['agent_id'] as String,
      streamId: json['stream_id'] as String,
    );

Map<String, dynamic> _$StreamStartMsgToJson(StreamStartMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'conversation_id': instance.conversationId,
      'agent_id': instance.agentId,
      'stream_id': instance.streamId,
    };

ContentDeltaMsg _$ContentDeltaMsgFromJson(Map<String, dynamic> json) =>
    ContentDeltaMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'content_delta',
      streamId: json['stream_id'] as String,
      delta: json['delta'] as String,
    );

Map<String, dynamic> _$ContentDeltaMsgToJson(ContentDeltaMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'stream_id': instance.streamId,
      'delta': instance.delta,
    };

ToolStartMsg _$ToolStartMsgFromJson(Map<String, dynamic> json) => ToolStartMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'tool_start',
      streamId: json['stream_id'] as String,
      tool: json['tool'] as String,
      label: json['label'] as String,
    );

Map<String, dynamic> _$ToolStartMsgToJson(ToolStartMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'stream_id': instance.streamId,
      'tool': instance.tool,
      'label': instance.label,
    };

ToolEndMsg _$ToolEndMsgFromJson(Map<String, dynamic> json) => ToolEndMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'tool_end',
      streamId: json['stream_id'] as String,
      tool: json['tool'] as String,
    );

Map<String, dynamic> _$ToolEndMsgToJson(ToolEndMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'stream_id': instance.streamId,
      'tool': instance.tool,
    };

StreamEndMsg _$StreamEndMsgFromJson(Map<String, dynamic> json) => StreamEndMsg(
      v: (json['v'] as num).toInt(),
      type: json['type'] as String? ?? 'stream_end',
      streamId: json['stream_id'] as String,
      tokens: (json['tokens'] as num).toInt(),
      durationMs: (json['duration_ms'] as num).toInt(),
    );

Map<String, dynamic> _$StreamEndMsgToJson(StreamEndMsg instance) =>
    <String, dynamic>{
      'v': instance.v,
      'type': instance.type,
      'stream_id': instance.streamId,
      'tokens': instance.tokens,
      'duration_ms': instance.durationMs,
    };
