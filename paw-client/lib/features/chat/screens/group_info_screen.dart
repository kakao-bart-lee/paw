import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../profile/widgets/avatar_widget.dart';
import '../providers/chat_provider.dart';
import '../../../core/theme/app_theme.dart';

class GroupInfoScreen extends ConsumerStatefulWidget {
  final String conversationId;

  const GroupInfoScreen({super.key, required this.conversationId});

  @override
  ConsumerState<GroupInfoScreen> createState() => _GroupInfoScreenState();
}

class _GroupInfoScreenState extends ConsumerState<GroupInfoScreen> {
  // Mock members for Phase 2
  final List<Map<String, dynamic>> _mockMembers = [
    {'id': 'user_1', 'name': '나 (방장)', 'role': 'owner', 'avatarUrl': null},
    {'id': 'user_2', 'name': '김철수', 'role': 'member', 'avatarUrl': null},
    {'id': 'user_3', 'name': '이영희', 'role': 'member', 'avatarUrl': null},
  ];

  void _showStubSnackBar(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  @override
  Widget build(BuildContext context) {
    final conversations = ref.watch(conversationsNotifierProvider);
    final conversation = conversations.firstWhere(
      (c) => c.id == widget.conversationId,
      orElse: () => throw Exception('Conversation not found'),
    );

    return Scaffold(
      appBar: AppBar(
        title: const Text('그룹 정보'),
        actions: [
          IconButton(
            icon: const Icon(Icons.edit),
            onPressed: () => _showStubSnackBar('그룹 이름 수정 기능은 준비 중입니다.'),
          ),
        ],
      ),
      body: Column(
        children: [
          const SizedBox(height: 24),
          // Group Avatar & Name
          AvatarWidget(
            displayName: conversation.name,
            imageUrl: conversation.avatarUrl,
            size: 80,
          ),
          const SizedBox(height: 16),
          Text(
            conversation.name,
            style: Theme.of(
              context,
            ).textTheme.headlineSmall?.copyWith(fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 8),
          // Member count chip
          Chip(
            label: Text('멤버 ${_mockMembers.length}명 / 100명'),
            backgroundColor: AppTheme.surface3,
          ),
          const SizedBox(height: 24),
          const Divider(height: 1),

          // Action Buttons
          Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: 16.0,
              vertical: 8.0,
            ),
            child: Row(
              children: [
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: () => _showStubSnackBar('멤버 추가 기능은 준비 중입니다.'),
                    icon: const Icon(Icons.person_add),
                    label: const Text('멤버 추가'),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: () => _showStubSnackBar('그룹 나가기 기능은 준비 중입니다.'),
                    icon: Icon(
                      Icons.exit_to_app,
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
                ),
              ],
            ),
          ),
          const Divider(height: 1),

          // Member List
          Expanded(
            child: ListView.builder(
              itemCount: _mockMembers.length,
              itemBuilder: (context, index) {
                final member = _mockMembers[index];
                final isOwner = member['role'] == 'owner';

                return ListTile(
                  leading: AvatarWidget(
                    displayName: member['name'] as String,
                    imageUrl: member['avatarUrl'] as String?,
                    size: 40,
                  ),
                  title: Text(member['name'] as String),
                  trailing: isOwner
                      ? Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 8,
                            vertical: 4,
                          ),
                          decoration: BoxDecoration(
                            color: AppTheme.primarySoft,
                            borderRadius: BorderRadius.circular(
                              AppTheme.radiusLg,
                            ),
                          ),
                          child: const Text(
                            '방장',
                            style: TextStyle(
                              fontSize: 12,
                              color: AppTheme.accent,
                            ),
                          ),
                        )
                      : null,
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
