import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../providers/auth_provider.dart';

class UsernameSetupScreen extends ConsumerStatefulWidget {
  const UsernameSetupScreen({super.key});

  @override
  ConsumerState<UsernameSetupScreen> createState() =>
      _UsernameSetupScreenState();
}

class _UsernameSetupScreenState extends ConsumerState<UsernameSetupScreen> {
  final TextEditingController _usernameController = TextEditingController();
  bool _discoverableByPhone = false;

  @override
  void initState() {
    super.initState();
    final authState = ref.read(authNotifierProvider);
    _usernameController.text = authState.username;
    _discoverableByPhone = authState.discoverableByPhone;
  }

  @override
  void dispose() {
    _usernameController.dispose();
    super.dispose();
  }

  bool get _isValidUsername {
    final value = _usernameController.text.trim();
    final usernamePattern = RegExp(r'^[a-z0-9_]{3,20}$');
    return usernamePattern.hasMatch(value);
  }

  Future<void> _submit() async {
    if (!_isValidUsername) return;

    await ref
        .read(authNotifierProvider.notifier)
        .completeUsernameSetup(
          username: _usernameController.text.trim(),
          discoverableByPhone: _discoverableByPhone,
        );

    if (!mounted) return;
    final state = ref.read(authNotifierProvider);
    if (state.step == AuthStep.authenticated) {
      context.go('/chat');
    }
  }

  void _skip() {
    ref.read(authNotifierProvider.notifier).skipUsernameSetup();
    context.go('/chat');
  }

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authNotifierProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('공개 아이디 설정')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'username 만들기',
                style: Theme.of(context).textTheme.headlineLarge,
              ),
              const SizedBox(height: 8),
              Text(
                '전화번호 대신 공유할 공개 아이디를 설정하세요. 지금은 건너뛰고 나중에 프로필에서 설정해도 됩니다.',
                style: Theme.of(context).textTheme.bodyMedium,
              ),
              const SizedBox(height: 24),
              TextField(
                controller: _usernameController,
                autocorrect: false,
                decoration: const InputDecoration(
                  labelText: 'username',
                  hintText: '예: paw_friend',
                  prefixText: '@',
                ),
                onChanged: (_) => setState(() {}),
              ),
              const SizedBox(height: 8),
              Text(
                '영문 소문자, 숫자, 밑줄만 사용 가능하며 3~20자여야 합니다.',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: 24),
              SwitchListTile.adaptive(
                value: _discoverableByPhone,
                contentPadding: EdgeInsets.zero,
                title: const Text('전화번호로 나를 찾기 허용'),
                subtitle: const Text('기본값은 꺼짐이며, 나중에 프로필에서 바꿀 수 있습니다.'),
                onChanged: (value) {
                  setState(() {
                    _discoverableByPhone = value;
                  });
                },
              ),
              if (authState.error != null) ...[
                const SizedBox(height: 12),
                Text(
                  authState.error!,
                  style: TextStyle(color: Theme.of(context).colorScheme.error),
                ),
              ],
              const SizedBox(height: 24),
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  onPressed: _isValidUsername && !authState.isLoading
                      ? _submit
                      : null,
                  child: authState.isLoading
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('계속'),
                ),
              ),
              const SizedBox(height: 12),
              SizedBox(
                width: double.infinity,
                child: OutlinedButton(
                  onPressed: authState.isLoading ? null : _skip,
                  child: const Text('나중에 할게요'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
