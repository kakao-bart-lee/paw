import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../providers/auth_provider.dart';

class LoginScreen extends ConsumerWidget {
  const LoginScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authNotifierProvider);

    return Scaffold(
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 420),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const Text(
                    '🐾',
                    textAlign: TextAlign.center,
                    style: TextStyle(fontSize: 64),
                  ),
                  const SizedBox(height: 16),
                  Text(
                    'Paw',
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.headlineLarge,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'AI-Native Messenger',
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                  const SizedBox(height: 32),
                  Text(
                    '로그인 방법 선택',
                    style: Theme.of(context).textTheme.headlineMedium,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    '기존 전화번호 OTP 로그인은 그대로 유지됩니다. 새 공개 아이디(username)는 로그인 후 설정할 수 있어요.',
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                  const SizedBox(height: 24),
                  FilledButton.icon(
                    onPressed: authState.isLoading
                        ? null
                        : () {
                            ref
                                .read(authNotifierProvider.notifier)
                                .showPhoneOtp();
                            context.go('/auth/phone');
                          },
                    icon: const Icon(Icons.sms_outlined),
                    label: const Text('전화번호로 계속'),
                  ),
                  const SizedBox(height: 12),
                  OutlinedButton.icon(
                    onPressed: null,
                    icon: const Icon(Icons.alternate_email_rounded),
                    label: const Text('username 로그인 준비 중'),
                  ),
                  const SizedBox(height: 16),
                  Text(
                    'day-1 호환성: 기존 사용자도 동일한 OTP 흐름으로 바로 접속할 수 있습니다.',
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.bodySmall,
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
