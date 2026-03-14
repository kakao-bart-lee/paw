import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';
import '../../../features/auth/providers/auth_provider.dart';
import '../providers/profile_provider.dart';
import '../widgets/avatar_widget.dart';

class MyProfileScreen extends ConsumerStatefulWidget {
  const MyProfileScreen({super.key});

  @override
  ConsumerState<MyProfileScreen> createState() => _MyProfileScreenState();
}

class _MyProfileScreenState extends ConsumerState<MyProfileScreen> {
  @override
  void initState() {
    super.initState();
    Future.microtask(() => ref.read(profileProvider.notifier).loadProfile());
  }

  Future<void> _showEditDialog(BuildContext context, String currentName) async {
    final controller = TextEditingController(text: currentName);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('이름 변경'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(labelText: '표시 이름'),
          autofocus: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: const Text('취소'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: const Text('확인'),
          ),
        ],
      ),
    );
    if (confirmed == true && controller.text.trim().isNotEmpty) {
      await ref
          .read(profileProvider.notifier)
          .updateProfile(controller.text.trim());
    }
  }

  @override
  Widget build(BuildContext context) {
    ref.listen<AuthState>(authNotifierProvider, (_, next) {
      if (next.step != AuthStep.authenticated && context.mounted) {
        context.go('/login');
      }
    });

    final profileState = ref.watch(profileProvider);

    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(title: const Text('내 프로필')),
      body: profileState.userAsync.when(
        loading: () => const _ProfileAsyncState(
          icon: Icons.person_outline_rounded,
          title: '프로필을 불러오는 중입니다',
          subtitle: '계정 정보와 보안 상태를 정리하고 있습니다.',
          loading: true,
        ),
        error: (e, _) => _ProfileAsyncState(
          icon: Icons.error_outline_rounded,
          title: '프로필을 불러오지 못했습니다',
          subtitle: '오류: $e',
          action: FilledButton.tonalIcon(
            onPressed: () => ref.read(profileProvider.notifier).loadProfile(),
            icon: const Icon(Icons.refresh_rounded),
            label: const Text('다시 시도'),
          ),
        ),
        data: (user) {
          final displayName = (user['display_name'] as String?) ?? '';
          final phone = (user['phone'] as String?) ?? '';
          final username = (user['username'] as String?) ?? '';
          final discoverableByPhone = user['discoverable_by_phone'] == true;
          final avatarUrl = user['avatar_url'] as String?;

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
                    const SizedBox(height: 8),
                    Text(
                      username.isNotEmpty ? '@$username' : 'username 미설정',
                      style: Theme.of(context).textTheme.bodyMedium,
                    ),
                    if (phone.isNotEmpty) ...[
                      const SizedBox(height: 6),
                      Text(phone, style: Theme.of(context).textTheme.bodySmall),
                    ],
                    const SizedBox(height: 12),
                    Text(
                      discoverableByPhone ? '전화번호 검색 허용됨' : '전화번호 검색 비공개',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    const SizedBox(height: 18),
                    Wrap(
                      spacing: 12,
                      runSpacing: 12,
                      alignment: WrapAlignment.center,
                      children: [
                        ElevatedButton.icon(
                          onPressed: profileState.isSaving
                              ? null
                              : () => _showEditDialog(context, displayName),
                          icon: const Icon(Icons.edit_rounded),
                          label: const Text('이름 변경'),
                        ),
                        OutlinedButton.icon(
                          onPressed: () => context.push('/auth/username-setup'),
                          icon: const Icon(Icons.alternate_email_rounded),
                          label: Text(
                            username.isEmpty ? 'username 설정' : 'username 수정',
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 18),
              Container(
                decoration: const BoxDecoration(
                  color: AppTheme.surface2,
                  borderRadius: BorderRadius.all(
                    Radius.circular(AppTheme.radiusXl),
                  ),
                  border: Border.fromBorderSide(
                    BorderSide(color: AppTheme.outline),
                  ),
                ),
                child: Column(
                  children: [
                    const _ProfileInfoRow(
                      icon: Icons.lock_outline_rounded,
                      title: '기본 보안',
                      subtitle: '종단간 암호화 대화와 기기 잠금을 함께 사용 중',
                    ),
                    const _ProfileInfoRow(
                      icon: Icons.auto_awesome_outlined,
                      title: 'Agent 활용',
                      subtitle: '권한은 대화별로 분리되어 안전하게 관리됩니다',
                    ),
                    _ProfileInfoRow(
                      icon: Icons.help_outline_rounded,
                      title: '지원 안내',
                      subtitle: 'visual QA와 Web/Desktop 지원 메모는 도움말 센터에서 확인',
                      trailing: TextButton(
                        onPressed: () => context.push('/settings/help'),
                        child: const Text('열기'),
                      ),
                      last: true,
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 20),
              OutlinedButton.icon(
                onPressed: () async {
                  await ref.read(authNotifierProvider.notifier).logout();
                  if (!context.mounted) return;
                  context.go('/login');
                },
                style: OutlinedButton.styleFrom(
                  foregroundColor: AppTheme.danger,
                  side: BorderSide(
                    color: AppTheme.danger.withValues(alpha: 0.4),
                  ),
                ),
                icon: const Icon(Icons.logout_rounded),
                label: const Text('로그아웃'),
              ),
            ],
          );
        },
      ),
    );
  }
}

class _ProfileAsyncState extends StatelessWidget {
  const _ProfileAsyncState({
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

class _ProfileInfoRow extends StatelessWidget {
  const _ProfileInfoRow({
    required this.icon,
    required this.title,
    required this.subtitle,
    this.trailing,
    this.last = false,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final Widget? trailing;
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
          if (trailing != null) ...[
            trailing!,
          ],
        ],
      ),
    );
  }
}
