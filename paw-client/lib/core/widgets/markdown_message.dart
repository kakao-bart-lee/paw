import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';
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
    final textColor = isMe ? Colors.white : theme.colorScheme.onSurface;
    
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
        color: isMe ? Colors.white70 : const Color(0xFF6C63FF),
        backgroundColor: isMe 
          ? Colors.white.withOpacity(0.15) 
          : const Color(0xFF252525),
      ),
      codeblockDecoration: BoxDecoration(
        color: const Color(0xFF1A1A1A),
        borderRadius: BorderRadius.circular(8),
      ),
      blockquoteDecoration: BoxDecoration(
        border: Border(
          left: BorderSide(
            color: isMe ? Colors.white54 : const Color(0xFF6C63FF),
            width: 3,
          ),
        ),
      ),
      blockquote: TextStyle(color: textColor.withOpacity(0.8)),
      listBullet: TextStyle(color: textColor),
      a: const TextStyle(
        color: Color(0xFF6C63FF),
        decoration: TextDecoration.underline,
      ),
    );
  }
}
