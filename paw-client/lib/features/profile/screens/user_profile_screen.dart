import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:get_it/get_it.dart';

import '../../../core/http/api_client.dart';
import '../widgets/avatar_widget.dart';

class UserProfileScreen extends ConsumerStatefulWidget {
  final String userId;

  const UserProfileScreen({super.key, required this.userId});

  @override
  ConsumerState<UserProfileScreen> createState() => _UserProfileScreenState();
}

class _UserProfileScreenState extends ConsumerState<UserProfileScreen> {
  AsyncValue<Map<String, dynamic>> _userAsync = const AsyncValue.loading();

  @override
  void initState() {
    super.initState();
    _loadUser();
  }

  Future<void> _loadUser() async {
    try {
      final result =
          await GetIt.instance<ApiClient>().searchUser(widget.userId);
      if (result == null) {
        setState(() {
          _userAsync = AsyncValue.error(
            '사용자를 찾을 수 없습니다',
            StackTrace.current,
          );
        });
      } else {
        setState(() {
          _userAsync = AsyncValue.data(result);
        });
      }
    } catch (e, st) {
      setState(() {
        _userAsync = AsyncValue.error(e, st);
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('프로필')),
      body: _userAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('오류: $e')),
        data: (user) {
          final displayName = (user['display_name'] as String?) ?? '';
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
                const SizedBox(height: 24),
                ElevatedButton(
                  onPressed: () {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('준비 중입니다')),
                    );
                  },
                  child: const Text('메시지 보내기'),
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}
