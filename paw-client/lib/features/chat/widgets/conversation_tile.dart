import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';
import '../models/conversation.dart';

class ConversationTile extends StatelessWidget {
  final Conversation conversation;
  final VoidCallback? onTap;

  const ConversationTile({
    super.key,
    required this.conversation,
    this.onTap,
  });

  String _formatTimestamp(DateTime time) {
    final now = DateTime.now();
    if (now.year == time.year && now.month == time.month && now.day == time.day) {
      return DateFormat('a h:mm', 'ko_KR').format(time);
    }
    return DateFormat('MM월 dd일', 'ko_KR').format(time);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    
    return ListTile(
      onTap: onTap ?? () {
        // Navigate to chat screen
        context.push('/chat/${conversation.id}');
      },
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      leading: CircleAvatar(
        radius: 24, // 48dp diameter
        backgroundColor: theme.colorScheme.surfaceVariant,
        backgroundImage: conversation.avatarUrl != null 
            ? NetworkImage(conversation.avatarUrl!) 
            : null,
        child: conversation.avatarUrl == null
            ? Text(
                conversation.name.isNotEmpty ? conversation.name[0] : '?',
                style: theme.textTheme.titleLarge?.copyWith(
                  color: theme.colorScheme.onSurface,
                ),
              )
            : null,
      ),
      title: Text(
        conversation.name,
        style: theme.textTheme.titleMedium?.copyWith(
          fontWeight: FontWeight.bold,
        ),
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: conversation.lastMessage != null
          ? Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                conversation.lastMessage!.content,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            )
          : null,
      trailing: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Text(
            _formatTimestamp(conversation.updatedAt),
            style: theme.textTheme.labelSmall,
          ),
          const SizedBox(height: 4),
          if (conversation.unreadCount > 0)
            Container(
              padding: const EdgeInsets.all(6),
              decoration: const BoxDecoration(
                color: Colors.red,
                shape: BoxShape.circle,
              ),
              child: Text(
                conversation.unreadCount > 99 ? '99+' : conversation.unreadCount.toString(),
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                ),
              ),
            )
          else
            const SizedBox(height: 22), // Placeholder to maintain alignment
        ],
      ),
    );
  }
}
