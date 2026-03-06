import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../../../core/theme/app_theme.dart';
import '../../../core/widgets/markdown_message.dart';
import '../models/message.dart';
import 'read_receipt_indicator.dart';
import 'media_message.dart';
import 'tool_indicator.dart';

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
                child: Column(
                  crossAxisAlignment: alignment,
                  children: [
                    if (isAgent && message.toolCalls.isNotEmpty)
                      Padding(
                        padding: const EdgeInsets.only(bottom: 8),
                        child: Wrap(
                          spacing: 4,
                          runSpacing: 4,
                          children: message.toolCalls
                              .map((tc) => ToolIndicator(
                                    toolName: tc.tool,
                                    label: tc.label,
                                    isComplete: true,
                                  ))
                              .toList(),
                        ),
                      ),
                    Container(
                      constraints: BoxConstraints(
                        maxWidth: MediaQuery.of(context).size.width * 0.75,
                      ),
                      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                      decoration: BoxDecoration(
                        color: bubbleColor,
                        borderRadius: borderRadius,
                      ),
                      child: message.mediaId != null
                          ? MediaMessage(
                              mediaId: message.mediaId!,
                              contentType: message.mediaType ?? 'application/octet-stream',
                              fileName: message.mediaFileName,
                              sizeBytes: message.mediaSizeBytes,
                              isMe: isMe,
                            )
                          : message.format == MessageFormat.markdown
                              ? MarkdownMessage(content: message.content, isMe: isMe)
                              : Text(
                                  message.content,
                                  style: theme.textTheme.bodyLarge?.copyWith(
                                    color: isMe ? Colors.white : theme.colorScheme.onSurface,
                                  ),
                                ),
                    ),
                    if (isAgent && message.blocks.isNotEmpty)
                      Padding(
                        padding: const EdgeInsets.only(top: 8),
                        child: Container(
                          constraints: BoxConstraints(
                            maxWidth: MediaQuery.of(context).size.width * 0.75,
                          ),
                          child: Column(
                            crossAxisAlignment: alignment,
                            children: message.blocks.map((block) {
                              if (block is CardBlock) {
                                return Card(
                                  margin: const EdgeInsets.only(bottom: 8),
                                  clipBehavior: Clip.antiAlias,
                                  child: Column(
                                    crossAxisAlignment: CrossAxisAlignment.start,
                                    children: [
                                      if (block.imageUrl != null)
                                        Image.network(
                                          block.imageUrl!,
                                          fit: BoxFit.cover,
                                          width: double.infinity,
                                          height: 150,
                                          errorBuilder: (context, error, stackTrace) =>
                                              const SizedBox(height: 150, child: Center(child: Icon(Icons.error))),
                                        ),
                                      Padding(
                                        padding: const EdgeInsets.all(12),
                                        child: Column(
                                          crossAxisAlignment: CrossAxisAlignment.start,
                                          children: [
                                            Text(
                                              block.title,
                                              style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
                                            ),
                                            if (block.description != null) ...[
                                              const SizedBox(height: 4),
                                              Text(block.description!),
                                            ],
                                          ],
                                        ),
                                      ),
                                    ],
                                  ),
                                );
                              } else if (block is ActionButtonsBlock) {
                                return Padding(
                                  padding: const EdgeInsets.only(bottom: 8),
                                  child: Wrap(
                                    spacing: 8,
                                    runSpacing: 8,
                                    children: block.buttons.map((btn) {
                                      return OutlinedButton(
                                        onPressed: () {
                                          // Action URL handling would go here
                                        },
                                        child: Text(btn.label),
                                      );
                                    }).toList(),
                                  ),
                                );
                              }
                              return const SizedBox.shrink();
                            }).toList(),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
              if (!isMe) const SizedBox(width: 8),
              if (!isMe) _buildTimestamp(theme),
            ],
          ),
          if (isMe)
            const Padding(
              padding: EdgeInsets.only(top: 2),
              child: ReadReceiptIndicator(status: ReadReceiptStatus.sent),
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
