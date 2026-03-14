import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/auth_provider.dart';
import '../widgets/otp_input_field.dart';

class OtpVerifyScreen extends ConsumerStatefulWidget {
  const OtpVerifyScreen({super.key});

  @override
  ConsumerState<OtpVerifyScreen> createState() => _OtpVerifyScreenState();
}

class _OtpVerifyScreenState extends ConsumerState<OtpVerifyScreen> {
  String _otpCode = '';
  int _secondsRemaining = 60;
  Timer? _timer;

  @override
  void initState() {
    super.initState();
    _startTimer();
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  void _startTimer() {
    setState(() {
      _secondsRemaining = 60;
    });
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (_secondsRemaining > 0) {
        setState(() {
          _secondsRemaining--;
        });
      } else {
        timer.cancel();
      }
    });
  }

  void _resendOtp() {
    final phone = ref.read(authNotifierProvider).phone;
    ref.read(authNotifierProvider.notifier).requestOtp(phone);
    _startTimer();
  }

  void _verifyOtp(String code) async {
    await ref.read(authNotifierProvider.notifier).verifyOtp(code);

    if (mounted) {
      final state = ref.read(authNotifierProvider);
      if (state.step == AuthStep.deviceName) {
        context.push('/auth/device-name');
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authNotifierProvider);
    final hasError = authState.error != null;

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const SizedBox(height: 16),
              Text('인증번호 입력', style: Theme.of(context).textTheme.headlineLarge),
              const SizedBox(height: 8),
              Text(
                '${authState.phone}로 전송된 6자리 코드를 입력하세요',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
              const SizedBox(height: 32),
              OtpInputField(
                length: 6,
                hasError: hasError,
                onChanged: (value) {
                  setState(() {
                    _otpCode = value;
                  });
                },
                onCompleted: (value) {
                  _verifyOtp(value);
                },
              ),
              if (hasError) ...[
                const SizedBox(height: 8),
                Text(
                  authState.error!,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.error,
                  ),
                ),
              ],
              const SizedBox(height: 32),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text(
                    _secondsRemaining > 0
                        ? '00:${_secondsRemaining.toString().padLeft(2, '0')}'
                        : '인증번호가 만료되었습니다',
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: _secondsRemaining > 0
                          ? Theme.of(context).colorScheme.secondary
                          : Theme.of(context).colorScheme.error,
                    ),
                  ),
                  const SizedBox(width: 16),
                  TextButton(
                    onPressed: _secondsRemaining == 0 ? _resendOtp : null,
                    child: const Text('재전송'),
                  ),
                ],
              ),
              const SizedBox(height: 32),
              SizedBox(
                width: double.infinity,
                height: 52,
                child: FilledButton(
                  onPressed: _otpCode.length == 6 && !authState.isLoading
                      ? () => _verifyOtp(_otpCode)
                      : null,
                  child: authState.isLoading
                      ? SizedBox(
                          width: 24,
                          height: 24,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Theme.of(context).colorScheme.onPrimary,
                          ),
                        )
                      : const Text(
                          '확인',
                          style: TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
