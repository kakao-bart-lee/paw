import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/widgets/messenger_avatar.dart';
import '../models/conversation.dart';

class ConversationTile extends StatelessWidget {
  const ConversationTile({
    super.key,
    required this.conversation,
    this.onTap,
    this.selected = false,
  });

  final Conversation conversation;
  final VoidCallback? onTap;
  final bool selected;

  String _formatTimestamp(DateTime time) {
    final now = DateTime.now();
    if (now.year == time.year &&
        now.month == time.month &&
        now.day == time.day) {
      return DateFormat('a h:mm', 'ko_KR').format(time);
    }
    return DateFormat('MM월 dd일', 'ko_KR').format(time);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final subtitle = conversation.lastMessage?.content ?? '새로운 대화를 시작해보세요';
    final isAgentConversation = conversation.agents.isNotEmpty;

    return Theme(
      data: Theme.of(context).copyWith(splashFactory: InkRipple.splashFactory),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 3),
        child: Material(
          color: selected ? AppTheme.primarySoft : Colors.transparent,
          borderRadius: BorderRadius.circular(8),
          child: InkWell(
            key: ValueKey('conversation-tile-${conversation.id}'),
            borderRadius: BorderRadius.circular(8),
            onTap:
                onTap ??
                () {
                  context.push('/chat/${conversation.id}');
                },
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 11),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: selected
                      ? AppTheme.accent.withValues(alpha: 0.22)
                      : AppTheme.outline.withValues(alpha: 0.24),
                ),
              ),
              child: Row(
                children: [
                  MessengerAvatar(
                    name: conversation.name,
                    imageUrl: conversation.avatarUrl,
                    size: 44,
                    isAgent: isAgentConversation,
                    isOnline: conversation.unreadCount > 0,
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Expanded(
                              child: Text(
                                conversation.name,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                                style: theme.textTheme.titleMedium?.copyWith(
                                  fontWeight: FontWeight.w700,
                                ),
                              ),
                            ),
                            if (conversation.isE2ee) ...[
                              Icon(
                                Icons.lock_rounded,
                                size: 14,
                                color: AppTheme.accent.withValues(alpha: 0.84),
                              ),
                              const SizedBox(width: 6),
                            ],
                            Text(
                              _formatTimestamp(conversation.updatedAt),
                              style: theme.textTheme.labelSmall,
                            ),
                          ],
                        ),
                        const SizedBox(height: 6),
                        Row(
                          children: [
                            if (isAgentConversation) ...[
                              Container(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 7,
                                  vertical: 3,
                                ),
                                decoration: BoxDecoration(
                                  color: AppTheme.primarySoft,
                                  borderRadius: BorderRadius.circular(5),
                                  border: Border.all(
                                    color: AppTheme.accent.withValues(
                                      alpha: 0.16,
                                    ),
                                  ),
                                ),
                                child: Text(
                                  'Agent',
                                  style: theme.textTheme.labelSmall?.copyWith(
                                    color: AppTheme.accent,
                                    fontWeight: FontWeight.w800,
                                  ),
                                ),
                              ),
                              const SizedBox(width: 8),
                            ],
                            Expanded(
                              child: Text(
                                subtitle,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                                style: theme.textTheme.bodySmall?.copyWith(
                                  color: selected
                                      ? AppTheme.primaryDeep
                                      : AppTheme.mutedText,
                                ),
                              ),
                            ),
                            const SizedBox(width: 8),
                            if (conversation.unreadCount > 0)
                              Container(
                                constraints: const BoxConstraints(minWidth: 20),
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 6,
                                  vertical: 4,
                                ),
                                decoration: BoxDecoration(
                                  color: isAgentConversation
                                      ? AppTheme.accent
                                      : AppTheme.surface4,
                                  borderRadius: BorderRadius.circular(5),
                                ),
                                child: Text(
                                  conversation.unreadCount > 99
                                      ? '99+'
                                      : conversation.unreadCount.toString(),
                                  textAlign: TextAlign.center,
                                  style: theme.textTheme.labelSmall?.copyWith(
                                    color: isAgentConversation
                                        ? AppTheme.background
                                        : AppTheme.strongText,
                                    fontWeight: FontWeight.w800,
                                  ),
                                ),
                              ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
