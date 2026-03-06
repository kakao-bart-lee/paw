import 'package:flutter/foundation.dart';

enum MessageFormat { markdown, plain }
enum MessageSender { me, other, agent }

@immutable
class ToolCallRecord {
  final String tool;
  final String label;
  const ToolCallRecord({required this.tool, required this.label});
}

class Message {
  final String id;
  final String conversationId;
  final String senderId;
  final String content;
  final MessageFormat format;
  final int seq;
  final DateTime createdAt;
  final bool isMe;
  final bool isAgent;
  final List<ToolCallRecord> toolCalls;
  
  final String? mediaId;
  final String? mediaUrl;
  final String? mediaType;
  final String? mediaFileName;
  final int? mediaSizeBytes;
  
  const Message({
    required this.id,
    required this.conversationId,
    required this.senderId,
    required this.content,
    required this.format,
    required this.seq,
    required this.createdAt,
    required this.isMe,
    required this.isAgent,
    this.toolCalls = const [],
    this.mediaId,
    this.mediaUrl,
    this.mediaType,
    this.mediaFileName,
    this.mediaSizeBytes,
  });
}
