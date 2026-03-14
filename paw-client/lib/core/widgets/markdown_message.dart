import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';
import '../theme/app_theme.dart';
import 'code_block.dart';

class MarkdownMessage extends StatelessWidget {
  final String content;
  final bool isMe; // affects text color
  
  const MarkdownMessage({
    super.key,
    required this.content,
    required this.isMe,
  });
  
  @override
  Widget build(BuildContext context) {
    return MarkdownBody(
      data: content,
      selectable: true,
      styleSheet: _buildStyleSheet(context),
      builders: {
        'code': CodeBlockBuilder(isMe: isMe),
      },
      onTapLink: (text, href, title) {
        // Phase 2: open URL in browser
        // Phase 1: do nothing
      },
    );
  }
  
  MarkdownStyleSheet _buildStyleSheet(BuildContext context) {
    final theme = Theme.of(context);
    final textColor = isMe ? AppTheme.background : theme.colorScheme.onSurface;
    
    return MarkdownStyleSheet(
      p: TextStyle(color: textColor, fontSize: 15, height: 1.4),
      strong: TextStyle(color: textColor, fontWeight: FontWeight.bold),
      em: TextStyle(color: textColor, fontStyle: FontStyle.italic),
      h1: TextStyle(color: textColor, fontSize: 20, fontWeight: FontWeight.bold),
      h2: TextStyle(color: textColor, fontSize: 18, fontWeight: FontWeight.bold),
      h3: TextStyle(color: textColor, fontSize: 16, fontWeight: FontWeight.bold),
      code: TextStyle(
        fontFamily: 'monospace',
        fontSize: 13,
        color: isMe ? AppTheme.background.withValues(alpha: 0.78) : AppTheme.accent,
        backgroundColor: isMe 
          ? AppTheme.background.withValues(alpha: 0.15)
          : AppTheme.surface3,
      ),
      codeblockDecoration: BoxDecoration(
        color: AppTheme.surface3,
        borderRadius: BorderRadius.circular(8),
      ),
      blockquoteDecoration: BoxDecoration(
        border: Border(
          left: BorderSide(
            color: isMe ? AppTheme.background.withValues(alpha: 0.54) : AppTheme.accent,
            width: 3,
          ),
        ),
      ),
      blockquote: TextStyle(color: textColor.withValues(alpha: 0.8)),
      listBullet: TextStyle(color: textColor),
      a: const TextStyle(
        color: AppTheme.accent,
        decoration: TextDecoration.underline,
      ),
    );
  }
}
