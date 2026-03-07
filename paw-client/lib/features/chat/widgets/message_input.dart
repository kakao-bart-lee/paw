import 'package:flutter/material.dart';
import 'media_picker.dart';

class MessageInput extends StatefulWidget {
  final ValueChanged<String> onSend;
  final bool canSend;
  final String? sendDisabledReason;

  const MessageInput({
    super.key,
    required this.onSend,
    this.canSend = true,
    this.sendDisabledReason,
  });

  @override
  State<MessageInput> createState() => _MessageInputState();
}

class _MessageInputState extends State<MessageInput> {
  final _controller = TextEditingController();
  bool _hasText = false;

  @override
  void initState() {
    super.initState();
    _controller.addListener(() {
      final hasText = _controller.text.trim().isNotEmpty;
      if (hasText != _hasText) {
        setState(() {
          _hasText = hasText;
        });
      }
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _handleSend() {
    if (!widget.canSend) {
      if (widget.sendDisabledReason != null) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(widget.sendDisabledReason!)));
      }
      return;
    }

    final text = _controller.text.trim();
    if (text.isNotEmpty) {
      widget.onSend(text);
      _controller.clear();
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      color: theme.colorScheme.surface,
      padding: EdgeInsets.only(
        left: 8,
        right: 8,
        top: 8,
        bottom: MediaQuery.of(context).viewPadding.bottom + 8,
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          IconButton(
            icon: const Icon(Icons.attach_file),
            color: theme.colorScheme.onSurfaceVariant,
            onPressed: () => MediaPicker.show(
              context,
              onFilePicked: (path, contentType) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('미디어 선택 기능은 곧 추가됩니다')),
                );
              },
            ),
            tooltip: '첨부파일',
          ),
          Expanded(
            child: TextField(
              key: const ValueKey('chat-message-input'),
              controller: _controller,
              minLines: 1,
              maxLines: 5,
              textInputAction: TextInputAction.newline,
              decoration: InputDecoration(
                hintText: '메시지 입력...',
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 10,
                ),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(20),
                  borderSide: BorderSide.none,
                ),
                filled: true,
                fillColor: theme.colorScheme.surfaceVariant,
              ),
            ),
          ),
          const SizedBox(width: 8),
          Padding(
            padding: const EdgeInsets.only(bottom: 2),
            child: Container(
              decoration: BoxDecoration(
                color: _hasText
                    ? theme.colorScheme.primary
                    : theme.colorScheme.surfaceVariant,
                shape: BoxShape.circle,
              ),
              child: IconButton(
                key: const ValueKey('chat-send-button'),
                icon: const Icon(Icons.arrow_upward),
                tooltip: '전송',
                color: _hasText
                    ? theme.colorScheme.onPrimary
                    : theme.colorScheme.onSurfaceVariant,
                onPressed: (_hasText && widget.canSend) ? _handleSend : null,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
