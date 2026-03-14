import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:get_it/get_it.dart';
import 'package:go_router/go_router.dart';

import '../../../core/http/api_client.dart';
import '../../../core/theme/app_theme.dart';
import '../../chat/providers/chat_provider.dart';
import '../widgets/avatar_widget.dart';

class UserProfileScreen extends ConsumerStatefulWidget {
  const UserProfileScreen({super.key, required this.userId});

  final String userId;

  @override
  ConsumerState<UserProfileScreen> createState() => _UserProfileScreenState();
}

class _UserProfileScreenState extends ConsumerState<UserProfileScreen> {
  AsyncValue<Map<String, dynamic>> _userAsync = const AsyncValue.loading();
  bool _startingConversation = false;

  @override
  void initState() {
    super.initState();
    _loadUser();
  }

  Future<void> _loadUser() async {
    setState(() {
      _userAsync = const AsyncValue.loading();
    });

    try {
      final apiClient = GetIt.instance<ApiClient>();
      Map<String, dynamic>? result;

      try {
        result = await apiClient.getUserById(widget.userId);
      } on ApiException catch (error) {
        if (error.statusCode != 404) rethrow;

        final rawLookup = widget.userId.trim();
        final usernameLookup = rawLookup.startsWith('@')
            ? rawLookup.substring(1)
            : rawLookup;
        final looksLikePhone = rawLookup.startsWith('+');

        result = await apiClient.searchUser(
          username: looksLikePhone ? null : usernameLookup,
          phone: looksLikePhone ? rawLookup : null,
        );
      }

      if (result == null) {
        setState(() {
          _userAsync = AsyncValue.error('사용자를 찾을 수 없습니다', StackTrace.current);
        });
      } else {
        setState(() {
          _userAsync = AsyncValue.data(result!);
        });
      }
    } catch (e, st) {
      setState(() {
        _userAsync = AsyncValue.error(e, st);
      });
    }
  }

  Future<void> _startConversation(Map<String, dynamic> user) async {
    if (_startingConversation) return;
    setState(() => _startingConversation = true);

    try {
      final apiClient = GetIt.instance<ApiClient>();
      final displayName = (user['display_name'] as String?)?.trim();
      final targetUserId = (user['id'] ?? widget.userId).toString();
      final created = await apiClient.createConversation([
        targetUserId,
      ], name: displayName);
      final nestedConversation = created['conversation'];
      final nestedId = nestedConversation is Map
          ? nestedConversation['id']
          : null;
      final conversationId =
          (created['id'] ?? created['conversation_id'] ?? nestedId)?.toString();

      if (conversationId == null || conversationId.isEmpty) {
        throw StateError('대화 ID를 찾을 수 없습니다.');
      }

      await ref.read(conversationsNotifierProvider.notifier).refresh();
      if (!mounted) return;
      context.go('/chat/$conversationId');
    } catch (error) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('대화를 시작하지 못했습니다: $error')));
    } finally {
      if (mounted) {
        setState(() => _startingConversation = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(title: const Text('프로필')),
      body: _userAsync.when(
        loading: () => const _UserProfileAsyncState(
          icon: Icons.person_search_rounded,
          title: '프로필을 찾는 중입니다',
          subtitle: '대화 가능 여부와 사용자 정보를 불러오고 있습니다.',
          loading: true,
        ),
        error: (e, _) => _UserProfileAsyncState(
          icon: Icons.error_outline_rounded,
          title: '프로필을 불러오지 못했습니다',
          subtitle: '$e',
          action: FilledButton.tonalIcon(
            onPressed: _loadUser,
            icon: const Icon(Icons.refresh_rounded),
            label: const Text('다시 시도'),
          ),
        ),
        data: (user) {
          final displayName = (user['display_name'] as String?) ?? '';
          final avatarUrl = user['avatar_url'] as String?;
          final username = (user['username'] as String?) ?? '';

          return ListView(
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
                      imageUrl: avatarUrl,
                      displayName: displayName,
                      size: 88,
                    ),
                    const SizedBox(height: 18),
                    Text(
                      displayName.isNotEmpty ? displayName : '(이름 없음)',
                      style: Theme.of(context).textTheme.headlineMedium,
                    ),
                    if (username.isNotEmpty) ...[
                      const SizedBox(height: 8),
                      Text(
                        '@$username',
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                    ],
                    const SizedBox(height: 12),
                    Text(
                      '안전한 메시지 전송이 가능한 연락처',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    const SizedBox(height: 18),
                    ElevatedButton.icon(
                      onPressed: _startingConversation
                          ? null
                          : () => _startConversation(user),
                      icon: _startingConversation
                          ? const SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Icon(Icons.chat_bubble_outline_rounded),
                      label: Text(
                        _startingConversation ? '대화 준비 중...' : '메시지 보내기',
                      ),
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
                child: const Column(
                  children: [
                    _UserProfileInfoRow(
                      icon: Icons.verified_user_outlined,
                      title: '안전한 연락처',
                      subtitle: '대화 생성 전 사용자 검색과 권한 흐름을 한 번 더 확인합니다.',
                    ),
                    _UserProfileInfoRow(
                      icon: Icons.auto_awesome_outlined,
                      title: 'Agent 분리',
                      subtitle: '개인 프로필과 Agent 대화는 분리된 맥락으로 유지됩니다.',
                      last: true,
                    ),
                  ],
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}

class _UserProfileAsyncState extends StatelessWidget {
  const _UserProfileAsyncState({
    required this.icon,
    required this.title,
    required this.subtitle,
    this.loading = false,
    this.action,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final bool loading;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 420),
          child: Container(
            width: double.infinity,
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: AppTheme.surface2,
              borderRadius: BorderRadius.circular(AppTheme.radiusXl),
              border: Border.all(color: AppTheme.outline),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 64,
                  height: 64,
                  decoration: BoxDecoration(
                    color: AppTheme.surface3,
                    borderRadius: BorderRadius.circular(AppTheme.radiusXl),
                    border: Border.all(color: AppTheme.outline),
                  ),
                  child: loading
                      ? const Padding(
                          padding: EdgeInsets.all(18),
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : Icon(icon, color: AppTheme.mutedText),
                ),
                const SizedBox(height: 16),
                Text(title, style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 8),
                Text(
                  subtitle,
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
                if (action != null) ...[const SizedBox(height: 16), action!],
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _UserProfileInfoRow extends StatelessWidget {
  const _UserProfileInfoRow({
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
