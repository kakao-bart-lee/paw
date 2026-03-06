import 'package:flutter/foundation.dart';

enum MessageFormat { markdown, plain }
enum MessageSender { me, other, agent }

@immutable
class ToolCallRecord {
  final String tool;
  final String label;
  const ToolCallRecord({required this.tool, required this.label});
}

sealed class MessageBlock {
  const MessageBlock();
  
  factory MessageBlock.fromJson(Map<String, dynamic> json) {
    final type = json['type'] as String?;
    if (type == 'card') {
      return CardBlock(
        title: json['title'] as String? ?? '',
        description: json['description'] as String?,
        imageUrl: json['imageUrl'] as String?,
      );
    } else if (type == 'actions') {
      final buttonsList = json['buttons'] as List<dynamic>? ?? [];
      final buttons = buttonsList.map((b) {
        final bMap = b as Map<String, dynamic>;
        return ActionButton(
          label: bMap['label'] as String? ?? '',
          actionUrl: bMap['actionUrl'] as String?,
        );
      }).toList();
      return ActionButtonsBlock(buttons: buttons);
    }
    throw ArgumentError('Unknown block type: $type');
  }
}

class CardBlock extends MessageBlock {
  final String title;
  final String? description;
  final String? imageUrl;
  const CardBlock({required this.title, this.description, this.imageUrl});
}

class ActionButtonsBlock extends MessageBlock {
  final List<ActionButton> buttons;
  const ActionButtonsBlock({required this.buttons});
}

class ActionButton {
  final String label;
  final String? actionUrl;
  const ActionButton({required this.label, this.actionUrl});
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
  final List<MessageBlock> blocks;
  
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
    this.blocks = const [],
    this.mediaId,
    this.mediaUrl,
    this.mediaType,
    this.mediaFileName,
    this.mediaSizeBytes,
  });
}
