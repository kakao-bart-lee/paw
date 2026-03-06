import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../../../core/theme/app_theme.dart';
import '../models/message.dart';

class MessageBubble extends StatelessWidget {
  final Message message;

  const MessageBubble({
    super.key,
    required this.message,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isMe = message.isMe;
    final isAgent = message.isAgent;

    Color bubbleColor;
    if (isMe) {
      bubbleColor = AppTheme.sentBubbleDark;
    } else if (isAgent) {
      bubbleColor = AppTheme.agentBubbleDark;
    } else {
      bubbleColor = AppTheme.receivedBubbleDark;
    }

    final alignment = isMe ? CrossAxisAlignment.end : CrossAxisAlignment.start;
    final borderRadius = BorderRadius.only(
      topLeft: const Radius.circular(18),
      topRight: const Radius.circular(18),
      bottomLeft: Radius.circular(isMe ? 18 : 4),
      bottomRight: Radius.circular(isMe ? 4 : 18),
    );

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: Column(
        crossAxisAlignment: alignment,
        children: [
          if (isAgent)
            Padding(
              padding: const EdgeInsets.only(bottom: 4, left: 4),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Text('🤖', style: TextStyle(fontSize: 14)),
                  const SizedBox(width: 4),
                  Text(
                    'AI Agent',
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
            ),
          Row(
            mainAxisAlignment: isMe ? MainAxisAlignment.end : MainAxisAlignment.start,
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              if (isMe) _buildTimestamp(theme),
              if (isMe) const SizedBox(width: 8),
              Flexible(
                child: Container(
                  constraints: BoxConstraints(
                    maxWidth: MediaQuery.of(context).size.width * 0.75,
                  ),
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                  decoration: BoxDecoration(
                    color: bubbleColor,
                    borderRadius: borderRadius,
                  ),
                  child: Text(
                    message.content,
                    style: theme.textTheme.bodyLarge?.copyWith(
                      color: Colors.white,
                    ),
                  ),
                ),
              ),
              if (!isMe) const SizedBox(width: 8),
              if (!isMe) _buildTimestamp(theme),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildTimestamp(ThemeData theme) {
    return Text(
      DateFormat('a h:mm', 'ko_KR').format(message.createdAt),
      style: theme.textTheme.labelSmall?.copyWith(
        fontSize: 10,
      ),
    );
  }
}
