import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/widgets/markdown_message.dart';
import '../../../core/widgets/messenger_avatar.dart';
import '../models/message.dart';
import 'media_message.dart';
import 'read_receipt_indicator.dart';
import 'tool_indicator.dart';

class MessageBubble extends StatelessWidget {
  const MessageBubble({super.key, required this.message});

  final Message message;

  @override
  Widget build(BuildContext context) {
    final isMe = message.isMe;
    final isAgent = message.isAgent;
    final theme = Theme.of(context);
    final bubbleColor = isMe
        ? AppTheme.sentBubbleDark
        : isAgent
        ? AppTheme.agentBubbleDark
        : AppTheme.receivedBubbleDark;

    final textColor = isMe ? AppTheme.background : theme.colorScheme.onSurface;
    final bubbleBorder = isMe
        ? AppTheme.accent.withValues(alpha: 0.16)
        : isAgent
        ? AppTheme.accent.withValues(alpha: 0.18)
        : AppTheme.outline.withValues(alpha: 0.65);
    final borderRadius = BorderRadius.only(
      topLeft: const Radius.circular(8),
      topRight: const Radius.circular(8),
      bottomLeft: Radius.circular(isMe ? 8 : 3),
      bottomRight: Radius.circular(isMe ? 3 : 8),
    );

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      child: Row(
        mainAxisAlignment: isMe
            ? MainAxisAlignment.end
            : MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          if (!isMe) ...[
            Padding(
              padding: const EdgeInsets.only(right: 8, bottom: 18),
              child: MessengerAvatar(
                name: isAgent ? 'AI' : '상대방',
                size: 30,
                isAgent: isAgent,
                showPresence: false,
              ),
            ),
          ],
          Flexible(
            child: Column(
              crossAxisAlignment: isMe
                  ? CrossAxisAlignment.end
                  : CrossAxisAlignment.start,
              children: [
                if (isAgent)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 6, left: 2),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          Icons.auto_awesome_rounded,
                          size: 13,
                          color: AppTheme.accent.withValues(alpha: 0.95),
                        ),
                        const SizedBox(width: 4),
                        Text(
                          'AI 응답',
                          style: theme.textTheme.labelSmall?.copyWith(
                            color: AppTheme.accent,
                            fontWeight: FontWeight.w800,
                          ),
                        ),
                      ],
                    ),
                  ),
                if (isAgent && message.toolCalls.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Wrap(
                      spacing: 6,
                      runSpacing: 6,
                      children: message.toolCalls
                          .map(
                            (tc) => ToolIndicator(
                              toolName: tc.tool,
                              label: tc.label,
                              isComplete: true,
                            ),
                          )
                          .toList(),
                    ),
                  ),
                Container(
                  constraints: BoxConstraints(
                    maxWidth: MediaQuery.sizeOf(context).width * 0.72,
                  ),
                  padding: const EdgeInsets.symmetric(
                    horizontal: 15,
                    vertical: 12,
                  ),
                  decoration: BoxDecoration(
                    color: bubbleColor,
                    borderRadius: borderRadius,
                    border: Border.all(color: bubbleBorder),
                  ),
                  child: message.mediaId != null
                      ? MediaMessage(
                          mediaId: message.mediaId!,
                          contentType:
                              message.mediaType ?? 'application/octet-stream',
                          fileName: message.mediaFileName,
                          sizeBytes: message.mediaSizeBytes,
                          isMe: isMe,
                        )
                      : message.format == MessageFormat.markdown
                      ? MarkdownMessage(content: message.content, isMe: isMe)
                      : Text(
                          message.content,
                          style: theme.textTheme.bodyMedium?.copyWith(
                            color: textColor,
                            height: 1.5,
                          ),
                        ),
                ),
                if (isAgent && message.blocks.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 8),
                    child: ConstrainedBox(
                      constraints: BoxConstraints(
                        maxWidth: MediaQuery.sizeOf(context).width * 0.72,
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: message.blocks.map((block) {
                          if (block is CardBlock) {
                            return Card(
                              margin: const EdgeInsets.only(bottom: 8),
                              clipBehavior: Clip.antiAlias,
                              color: AppTheme.surface2,
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  if (block.imageUrl != null)
                                    Image.network(
                                      block.imageUrl!,
                                      fit: BoxFit.cover,
                                      width: double.infinity,
                                      height: 150,
                                      errorBuilder: (_, error, stackTrace) =>
                                          const SizedBox(
                                            height: 150,
                                            child: Center(
                                              child: Icon(
                                                Icons.error_outline_rounded,
                                              ),
                                            ),
                                          ),
                                    ),
                                  Padding(
                                    padding: const EdgeInsets.all(14),
                                    child: Column(
                                      crossAxisAlignment:
                                          CrossAxisAlignment.start,
                                      children: [
                                        Text(
                                          block.title,
                                          style: theme.textTheme.titleMedium,
                                        ),
                                        if (block.description != null) ...[
                                          const SizedBox(height: 6),
                                          Text(
                                            block.description!,
                                            style: theme.textTheme.bodySmall,
                                          ),
                                        ],
                                      ],
                                    ),
                                  ),
                                ],
                              ),
                            );
                          }

                          if (block is ActionButtonsBlock) {
                            return Padding(
                              padding: const EdgeInsets.only(bottom: 8),
                              child: Wrap(
                                spacing: 8,
                                runSpacing: 8,
                                children: block.buttons
                                    .map(
                                      (btn) => OutlinedButton(
                                        onPressed: () {},
                                        child: Text(btn.label),
                                      ),
                                    )
                                    .toList(),
                              ),
                            );
                          }

                          return const SizedBox.shrink();
                        }).toList(),
                      ),
                    ),
                  ),
                Padding(
                  padding: const EdgeInsets.only(top: 5, left: 4, right: 4),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        DateFormat('a h:mm', 'ko_KR').format(message.createdAt),
                        style: theme.textTheme.labelSmall,
                      ),
                      if (isMe) ...[
                        const SizedBox(width: 6),
                        const ReadReceiptIndicator(
                          status: ReadReceiptStatus.sent,
                        ),
                      ],
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
