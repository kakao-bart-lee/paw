import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/legacy.dart' as legacy;

import '../models/message.dart';

enum ResourceLoadState { loading, ready, error }

class SendMessageResult {
  final bool ok;
  final String? message;

  const SendMessageResult._({required this.ok, this.message});

  const SendMessageResult.success() : this._(ok: true);

  const SendMessageResult.failure(String message)
    : this._(ok: false, message: message);
}

class ToolRecord {
  final String tool;
  final String label;
  ToolRecord({required this.tool, required this.label});
}

class StreamingMessage {
  final String streamId;
  final String conversationId;
  final String agentId;
  final ValueNotifier<String> contentNotifier;
  bool isComplete;
  String? currentTool;
  String? currentToolLabel;
  bool toolComplete;
  final List<ToolRecord> toolHistory = [];

  StreamingMessage({
    required this.streamId,
    required this.conversationId,
    required this.agentId,
    String initialContent = '',
    this.isComplete = false,
    this.currentTool,
    this.currentToolLabel,
    this.toolComplete = false,
  }) : contentNotifier = ValueNotifier(initialContent);

  String get content => contentNotifier.value;
  set content(String value) => contentNotifier.value = value;

  void dispose() {
    contentNotifier.dispose();
  }
}

MessageFormat toMessageFormat(String format) {
  return format.toLowerCase() == 'markdown'
      ? MessageFormat.markdown
      : MessageFormat.plain;
}

Message messageFromJson(Map<String, dynamic> json) {
  return Message(
    id: (json['id'] ?? '').toString(),
    conversationId: (json['conversation_id'] ?? '').toString(),
    senderId: (json['sender_id'] ?? '').toString(),
    content: (json['content'] ?? '').toString(),
    format: toMessageFormat((json['format'] ?? 'plain').toString()),
    seq: (json['seq'] as num?)?.toInt() ?? 0,
    createdAt:
        DateTime.tryParse((json['created_at'] ?? '').toString()) ??
        DateTime.now(),
    isMe: false,
    isAgent: false,
  );
}

// Shared load/error state providers for conversations
final conversationsLoadStateProvider = legacy.StateProvider<ResourceLoadState>(
  (ref) => ResourceLoadState.loading,
);

final conversationsErrorProvider = legacy.StateProvider<String?>((ref) => null);

// Shared load/error state providers for messages (per conversation)
final messagesLoadStateProvider =
    legacy.StateProvider.family<ResourceLoadState, String>(
      (ref, _) => ResourceLoadState.loading,
    );

final messagesErrorProvider = legacy.StateProvider.family<String?, String>(
  (ref, _) => null,
);
