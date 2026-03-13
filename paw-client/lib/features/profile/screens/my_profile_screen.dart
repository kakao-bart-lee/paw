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
        context.go('/auth/phone');
      }
    });

    final profileState = ref.watch(profileProvider);

    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(title: const Text('내 프로필')),
      body: profileState.userAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('오류: $e')),
        data: (user) {
          final displayName = (user['display_name'] as String?) ?? '';
          final phone = (user['phone'] as String?) ?? '';
          final avatarUrl = user['avatar_url'] as String?;

          return ListView(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
            children: [
              Container(
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: AppTheme.surface2,
                  borderRadius: BorderRadius.circular(28),
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
                    Text(phone, style: Theme.of(context).textTheme.bodySmall),
                    const SizedBox(height: 18),
                    ElevatedButton.icon(
                      onPressed: profileState.isSaving
                          ? null
                          : () => _showEditDialog(context, displayName),
                      icon: const Icon(Icons.edit_rounded),
                      label: const Text('이름 변경'),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 18),
              Container(
                decoration: BoxDecoration(
                  color: AppTheme.surface2,
                  borderRadius: BorderRadius.circular(24),
                  border: Border.all(color: AppTheme.outline),
                ),
                child: Column(
                  children: const [
                    _ProfileInfoRow(
                      icon: Icons.lock_outline_rounded,
                      title: '기본 보안',
                      subtitle: '종단간 암호화 대화와 기기 잠금을 함께 사용 중',
                    ),
                    _ProfileInfoRow(
                      icon: Icons.auto_awesome_outlined,
                      title: 'Agent 활용',
                      subtitle: '권한은 대화별로 분리되어 안전하게 관리됩니다',
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
                  side: const BorderSide(color: Color(0x55FF7A7A)),
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

class _ProfileInfoRow extends StatelessWidget {
  const _ProfileInfoRow({
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
              borderRadius: BorderRadius.circular(14),
            ),
            child: Icon(icon, color: AppTheme.primary),
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
