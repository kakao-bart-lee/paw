import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';
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
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surface,
                  borderRadius: BorderRadius.circular(AppTheme.radiusMd),
                  border: Border.all(color: AppTheme.outline),
                ),
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Row(
                        children: [
                          Container(
                            width: 28,
                            height: 28,
                            decoration: BoxDecoration(
                              color: AppTheme.primarySoft,
                              borderRadius: BorderRadius.circular(
                                AppTheme.radiusXs,
                              ),
                              border: Border.all(
                                color: AppTheme.accent.withValues(alpha: 0.28),
                              ),
                            ),
                            child: Icon(
                              Icons.north_east_rounded,
                              size: 16,
                              color: AppTheme.accent,
                            ),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Container(
                              height: 1,
                              color: AppTheme.outline,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 18),
                      Text(
                        'Paw',
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),
                      const SizedBox(height: 6),
                      Text(
                        'AI-Native Messenger',
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                      const SizedBox(height: 28),
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
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
