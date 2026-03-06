import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';
import 'tool_indicator.dart';

class StreamBubble extends StatefulWidget {
  final String streamId;
  final ValueNotifier<String> contentNotifier;
  final bool isComplete;
  final String? toolName;
  final String? toolLabel;
  final bool toolComplete;

  const StreamBubble({
    super.key,
    required this.streamId,
    required this.contentNotifier,
    required this.isComplete,
    this.toolName,
    this.toolLabel,
    this.toolComplete = false,
  });

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
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        margin: const EdgeInsets.only(bottom: 16, left: 16, right: 64),
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: const Color(0xFF1E2A3A),
          borderRadius: BorderRadius.circular(16).copyWith(
            bottomLeft: const Radius.circular(4),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (widget.toolName != null && widget.toolLabel != null) ...[
              ToolIndicator(
                toolName: widget.toolName!,
                label: widget.toolLabel!,
                isComplete: widget.toolComplete,
              ),
              const SizedBox(height: 8),
            ],
            ValueListenableBuilder<String>(
              valueListenable: widget.contentNotifier,
              builder: (context, content, child) {
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
                          p: const TextStyle(color: Colors.white, fontSize: 16),
                          code: const TextStyle(
                            color: Colors.white,
                            backgroundColor: Colors.black26,
                            fontFamily: 'monospace',
                          ),
                          codeblockDecoration: BoxDecoration(
                            color: Colors.black26,
                            borderRadius: BorderRadius.circular(8),
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
                            color: Colors.white,
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
    );
  }
}
