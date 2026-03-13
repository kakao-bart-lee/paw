import 'package:flutter/material.dart';

import '../../../core/theme/app_theme.dart';
import 'media_picker.dart';

class MessageInput extends StatefulWidget {
  const MessageInput({
    super.key,
    required this.onSend,
    this.canSend = true,
    this.sendDisabledReason,
  });

  final ValueChanged<String> onSend;
  final bool canSend;
  final String? sendDisabledReason;

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
        setState(() => _hasText = hasText);
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
    if (text.isEmpty) return;
    widget.onSend(text);
    _controller.clear();
  }

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: Theme.of(context).copyWith(splashFactory: InkRipple.splashFactory),
      child: Container(
        decoration: BoxDecoration(
          color: AppTheme.surface2,
          border: Border(
            top: BorderSide(color: AppTheme.outline.withValues(alpha: 0.8)),
          ),
        ),
        padding: EdgeInsets.only(
          left: 12,
          right: 12,
          top: 12,
          bottom: MediaQuery.of(context).viewPadding.bottom + 10,
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            _RoundIconButton(
              icon: Icons.attach_file_rounded,
              tooltip: '첨부파일',
              onPressed: () => MediaPicker.show(
                context,
                onFilePicked: (_, __) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('미디어 선택 기능은 곧 추가됩니다')),
                  );
                },
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: AppTheme.surface3,
                  borderRadius: BorderRadius.circular(24),
                  border: Border.all(color: AppTheme.outline),
                ),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Expanded(
                      child: TextField(
                        key: const ValueKey('chat-message-input'),
                        controller: _controller,
                        minLines: 1,
                        maxLines: 5,
                        textInputAction: TextInputAction.newline,
                        decoration: const InputDecoration(
                          hintText: '메시지 입력...',
                          border: InputBorder.none,
                          enabledBorder: InputBorder.none,
                          focusedBorder: InputBorder.none,
                          fillColor: Colors.transparent,
                          filled: false,
                        ),
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.only(right: 4, bottom: 4),
                      child: IconButton(
                        onPressed: () {},
                        icon: const Icon(Icons.sentiment_satisfied_alt_rounded),
                        color: AppTheme.mutedText,
                        tooltip: '이모지',
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(width: 8),
            Container(
              decoration: BoxDecoration(
                color: _hasText ? AppTheme.primary : AppTheme.surface3,
                shape: BoxShape.circle,
                border: Border.all(
                  color: _hasText ? AppTheme.primary : AppTheme.outline,
                ),
              ),
              child: IconButton(
                key: const ValueKey('chat-send-button'),
                icon: Icon(
                  _hasText ? Icons.arrow_upward : Icons.mic_none_rounded,
                ),
                tooltip: '전송',
                color: _hasText ? AppTheme.background : AppTheme.mutedText,
                onPressed: (_hasText && widget.canSend) ? _handleSend : null,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _RoundIconButton extends StatelessWidget {
  const _RoundIconButton({
    required this.icon,
    required this.tooltip,
    required this.onPressed,
  });

  final IconData icon;
  final String tooltip;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.surface3,
        shape: BoxShape.circle,
        border: Border.all(color: AppTheme.outline),
      ),
      child: IconButton(
        icon: Icon(icon),
        color: AppTheme.mutedText,
        tooltip: tooltip,
        onPressed: onPressed,
      ),
    );
  }
}
