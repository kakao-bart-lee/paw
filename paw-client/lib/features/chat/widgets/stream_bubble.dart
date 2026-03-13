import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/widgets/messenger_avatar.dart';
import 'tool_indicator.dart';

class StreamBubble extends StatefulWidget {
  const StreamBubble({
    super.key,
    required this.streamId,
    required this.contentNotifier,
    required this.isComplete,
    this.toolName,
    this.toolLabel,
    this.toolComplete = false,
  });

  final String streamId;
  final ValueNotifier<String> contentNotifier;
  final bool isComplete;
  final String? toolName;
  final String? toolLabel;
  final bool toolComplete;

  @override
  State<StreamBubble> createState() => _StreamBubbleState();
}

class _StreamBubbleState extends State<StreamBubble>
    with SingleTickerProviderStateMixin {
  late AnimationController _cursorController;

  @override
  void initState() {
    super.initState();
    _cursorController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 500),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _cursorController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          const Padding(
            padding: EdgeInsets.only(right: 10, bottom: 22),
            child: MessengerAvatar(
              name: 'AI',
              size: 32,
              isAgent: true,
              showPresence: false,
            ),
          ),
          Flexible(
            child: Container(
              margin: const EdgeInsets.only(right: 56),
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: AppTheme.agentBubbleDark,
                borderRadius: BorderRadius.circular(
                  24,
                ).copyWith(bottomLeft: const Radius.circular(8)),
                border: Border.all(
                  color: AppTheme.primary.withValues(alpha: 0.14),
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.auto_awesome_rounded,
                        size: 13,
                        color: AppTheme.primary.withValues(alpha: 0.9),
                      ),
                      const SizedBox(width: 4),
                      Text(
                        'AI 응답',
                        style: Theme.of(context).textTheme.labelSmall?.copyWith(
                          color: AppTheme.primary,
                          fontWeight: FontWeight.w800,
                        ),
                      ),
                    ],
                  ),
                  if (widget.toolName != null && widget.toolLabel != null) ...[
                    const SizedBox(height: 8),
                    ToolIndicator(
                      toolName: widget.toolName!,
                      label: widget.toolLabel!,
                      isComplete: widget.toolComplete,
                    ),
                  ],
                  const SizedBox(height: 8),
                  ValueListenableBuilder<String>(
                    valueListenable: widget.contentNotifier,
                    builder: (context, content, _) {
                      if (content.isEmpty && widget.isComplete) {
                        return const SizedBox.shrink();
                      }

                      return Row(
                        crossAxisAlignment: CrossAxisAlignment.end,
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Flexible(
                            child: MarkdownBody(
                              data: content,
                              styleSheet: MarkdownStyleSheet(
                                p: Theme.of(context).textTheme.bodyMedium
                                    ?.copyWith(color: AppTheme.strongText),
                                code: const TextStyle(
                                  color: AppTheme.strongText,
                                  backgroundColor: AppTheme.surface4,
                                  fontFamily: 'monospace',
                                ),
                                codeblockDecoration: BoxDecoration(
                                  color: AppTheme.surface4,
                                  borderRadius: BorderRadius.circular(12),
                                ),
                              ),
                            ),
                          ),
                          if (!widget.isComplete)
                            FadeTransition(
                              opacity: _cursorController,
                              child: const Text(
                                '▋',
                                style: TextStyle(
                                  color: AppTheme.primary,
                                  fontSize: 16,
                                ),
                              ),
                            ),
                        ],
                      );
                    },
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
