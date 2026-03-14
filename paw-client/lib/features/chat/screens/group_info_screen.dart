import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../profile/widgets/avatar_widget.dart';
import '../providers/chat_provider.dart';
import '../../../core/theme/app_theme.dart';
import '../../../core/widgets/feedback_state_card.dart';

class GroupInfoScreen extends ConsumerStatefulWidget {
  final String conversationId;

  const GroupInfoScreen({super.key, required this.conversationId});

  @override
  ConsumerState<GroupInfoScreen> createState() => _GroupInfoScreenState();
}

class _GroupInfoScreenState extends ConsumerState<GroupInfoScreen> {
  final List<_GroupMember> _mockMembers = const [
    _GroupMember(id: 'user_1', name: '나 (방장)', role: 'owner'),
    _GroupMember(id: 'user_2', name: '김철수', role: 'member'),
    _GroupMember(id: 'user_3', name: '이영희', role: 'member'),
  ];

  void _showStubSnackBar(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  @override
  Widget build(BuildContext context) {
    final conversations = ref.watch(conversationsNotifierProvider);
    final conversation = conversations
        .where((c) => c.id == widget.conversationId)
        .firstOrNull;

    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(
        title: const Text('채팅 상세'),
        actions: [
          if (conversation != null)
            IconButton(
              icon: const Icon(Icons.edit_rounded),
              onPressed: () => _showStubSnackBar('그룹 이름 수정 기능은 준비 중입니다.'),
            ),
        ],
      ),
      body: conversation == null
          ? _GroupInfoFeedbackState(
              icon: Icons.forum_outlined,
              title: '대화 정보를 찾지 못했습니다',
              subtitle: '목록을 새로고침한 뒤 다시 시도하거나 채팅 목록으로 돌아가세요.',
              action: FilledButton.tonalIcon(
                onPressed: () => context.go('/chat'),
                icon: const Icon(Icons.arrow_back_rounded),
                label: const Text('채팅으로 이동'),
              ),
            )
          : ListView(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
              children: [
                Container(
                  padding: const EdgeInsets.all(24),
                  decoration: BoxDecoration(
                    color: AppTheme.surface2,
                    borderRadius: BorderRadius.circular(AppTheme.radiusXl),
                    border: Border.all(color: AppTheme.outline),
                  ),
                  child: Column(
                    children: [
                      AvatarWidget(
                        displayName: conversation.name,
                        imageUrl: conversation.avatarUrl,
                        size: 80,
                      ),
                      const SizedBox(height: 16),
                      Text(
                        conversation.name,
                        style: Theme.of(context).textTheme.headlineSmall
                            ?.copyWith(fontWeight: FontWeight.bold),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 8),
                      Wrap(
                        spacing: 8,
                        runSpacing: 8,
                        alignment: WrapAlignment.center,
                        children: [
                          Chip(
                            label: Text('멤버 ${_mockMembers.length}명 / 100명'),
                          ),
                          if (conversation.isE2ee)
                            const Chip(label: Text('종단간 암호화')),
                          if (conversation.agents.isNotEmpty)
                            const Chip(label: Text('Agent 연동')),
                        ],
                      ),
                      const SizedBox(height: 18),
                      Row(
                        children: [
                          Expanded(
                            child: OutlinedButton.icon(
                              onPressed: () =>
                                  _showStubSnackBar('멤버 추가 기능은 준비 중입니다.'),
                              icon: const Icon(Icons.person_add_alt_rounded),
                              label: const Text('멤버 추가'),
                            ),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: OutlinedButton.icon(
                              onPressed: () => context.push('/settings/help'),
                              icon: const Icon(Icons.help_outline_rounded),
                              label: const Text('도움말'),
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 18),
                Container(
                  decoration: BoxDecoration(
                    color: AppTheme.surface2,
                    borderRadius: BorderRadius.circular(AppTheme.radiusXl),
                    border: Border.all(color: AppTheme.outline),
                  ),
                  child: Column(
                    children: [
                      _GroupInfoRow(
                        icon: Icons.shield_outlined,
                        title: '대화 상태',
                        subtitle: conversation.isE2ee
                            ? '보안 대화가 활성화되어 있습니다.'
                            : '표준 대화 상태입니다. 필요 시 보안 전환을 검토하세요.',
                      ),
                      const _GroupInfoRow(
                        icon: Icons.auto_awesome_outlined,
                        title: '제품 톤 체크',
                        subtitle:
                            '이 상세 화면은 검색·프로필·설정과 동일한 warm editorial 톤을 유지합니다.',
                        last: true,
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 18),
                Container(
                  decoration: BoxDecoration(
                    color: AppTheme.surface2,
                    borderRadius: BorderRadius.circular(AppTheme.radiusXl),
                    border: Border.all(color: AppTheme.outline),
                  ),
                  child: Column(
                    children: [
                      for (var index = 0; index < _mockMembers.length; index++)
                        _MemberRow(
                          member: _mockMembers[index],
                          last: index == _mockMembers.length - 1,
                        ),
                    ],
                  ),
                ),
                const SizedBox(height: 18),
                OutlinedButton.icon(
                  onPressed: () => _showStubSnackBar('그룹 나가기 기능은 준비 중입니다.'),
                  icon: Icon(
                    Icons.exit_to_app_rounded,
                    color: AppTheme.danger.withValues(alpha: 0.88),
                  ),
                  label: Text(
                    '그룹 나가기',
                    style: TextStyle(
                      color: AppTheme.danger.withValues(alpha: 0.88),
                    ),
                  ),
                  style: OutlinedButton.styleFrom(
                    side: BorderSide(
                      color: AppTheme.danger.withValues(alpha: 0.4),
                    ),
                  ),
                ),
              ],
            ),
    );
  }
}

class _GroupInfoFeedbackState extends StatelessWidget {
  const _GroupInfoFeedbackState({
    required this.icon,
    required this.title,
    required this.subtitle,
    this.action,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    return FeedbackStateCard(
      icon: icon,
      title: title,
      subtitle: subtitle,
      action: action,
    );
  }
}

class _GroupInfoRow extends StatelessWidget {
  const _GroupInfoRow({
    required this.icon,
    required this.title,
    required this.subtitle,
    this.last = false,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final bool last;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
      decoration: BoxDecoration(
        border: last
            ? null
            : const Border(bottom: BorderSide(color: AppTheme.outline)),
      ),
      child: Row(
        children: [
          Container(
            width: 38,
            height: 38,
            decoration: BoxDecoration(
              color: AppTheme.surface4,
              borderRadius: BorderRadius.circular(AppTheme.radiusLg),
            ),
            child: Icon(icon, color: AppTheme.accent),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(title, style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 4),
                Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _MemberRow extends StatelessWidget {
  const _MemberRow({required this.member, this.last = false});

  final _GroupMember member;
  final bool last;

  @override
  Widget build(BuildContext context) {
    final isOwner = member.role == 'owner';

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
      decoration: BoxDecoration(
        border: last
            ? null
            : const Border(bottom: BorderSide(color: AppTheme.outline)),
      ),
      child: Row(
        children: [
          AvatarWidget(
            displayName: member.name,
            imageUrl: null,
            size: 40,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              member.name,
              style: Theme.of(context).textTheme.titleMedium,
            ),
          ),
          if (isOwner)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: AppTheme.primarySoft,
                borderRadius: BorderRadius.circular(AppTheme.radiusLg),
              ),
              child: const Text(
                '방장',
                style: TextStyle(fontSize: 12, color: AppTheme.accent),
              ),
            ),
        ],
      ),
    );
  }
}

class _GroupMember {
  const _GroupMember({
    required this.id,
    required this.name,
    required this.role,
  });

  final String id;
  final String name;
  final String role;
}
