import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

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
    Future.microtask(
      () => ref.read(profileProvider.notifier).loadProfile(),
    );
  }

  Future<void> _showEditDialog(
    BuildContext context,
    String currentName,
  ) async {
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
    final profileState = ref.watch(profileProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('내 프로필')),
      body: profileState.userAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('오류: $e')),
        data: (user) {
          final displayName = (user['display_name'] as String?) ?? '';
          final phone = (user['phone'] as String?) ?? '';
          final avatarUrl = user['avatar_url'] as String?;

          return Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                const SizedBox(height: 24),
                AvatarWidget(
                  imageUrl: avatarUrl,
                  displayName: displayName,
                  size: 80,
                ),
                const SizedBox(height: 16),
                Text(
                  displayName.isNotEmpty ? displayName : '(이름 없음)',
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
                const SizedBox(height: 8),
                Text(
                  phone,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: Colors.grey,
                      ),
                ),
                const SizedBox(height: 24),
                ElevatedButton(
                  onPressed: profileState.isSaving
                      ? null
                      : () => _showEditDialog(context, displayName),
                  child: const Text('이름 변경'),
                ),
                const Spacer(),
                TextButton(
                  onPressed: () async {
                    await ref.read(authNotifierProvider.notifier).logout();
                    if (!context.mounted) return;
                    context.go('/login');
                  },
                  child: const Text(
                    '로그아웃',
                    style: TextStyle(color: Colors.red),
                  ),
                ),
                const SizedBox(height: 16),
              ],
            ),
          );
        },
      ),
    );
  }
}
