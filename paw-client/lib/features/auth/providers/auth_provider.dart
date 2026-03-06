import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'auth_provider.g.dart';

enum AuthStep { phoneInput, otpVerify, deviceName, authenticated }

class AuthState {
  final AuthStep step;
  final String phone;
  final String deviceName;
  final bool isLoading;
  final String? error;

  const AuthState({
    this.step = AuthStep.phoneInput,
    this.phone = '',
    this.deviceName = '',
    this.isLoading = false,
    this.error,
  });

  const AuthState.initial()
      : step = AuthStep.phoneInput,
        phone = '',
        deviceName = '',
        isLoading = false,
        error = null;

  AuthState copyWith({
    AuthStep? step,
    String? phone,
    String? deviceName,
    bool? isLoading,
    String? error,
  }) {
    return AuthState(
      step: step ?? this.step,
      phone: phone ?? this.phone,
      deviceName: deviceName ?? this.deviceName,
      isLoading: isLoading ?? this.isLoading,
      error: error ?? this.error,
    );
  }
}

@riverpod
class AuthNotifier extends _$AuthNotifier {
  @override
  AuthState build() => const AuthState.initial();

  Future<void> requestOtp(String phone) async {
    state = state.copyWith(isLoading: true, error: null);
    
    // Phase 1: simulate OTP request (no real HTTP yet)
    // T13 will wire up real HTTP client
    await Future.delayed(const Duration(seconds: 1));
    
    state = state.copyWith(
      step: AuthStep.otpVerify,
      phone: phone,
      isLoading: false,
    );
  }

  Future<void> verifyOtp(String code) async {
    state = state.copyWith(isLoading: true, error: null);
    
    // Phase 1: accept any 6-digit code (mock)
    // T13 will wire up real verification
    await Future.delayed(const Duration(seconds: 1));
    
    if (code.length == 6) {
      state = state.copyWith(
        step: AuthStep.deviceName,
        isLoading: false,
      );
    } else {
      state = state.copyWith(
        isLoading: false,
        error: 'Invalid OTP code',
      );
    }
  }

  Future<void> setDeviceName(String name) async {
    state = state.copyWith(isLoading: true, error: null);
    
    // Phase 1: mock device registration
    await Future.delayed(const Duration(seconds: 1));
    
    state = state.copyWith(
      step: AuthStep.authenticated,
      deviceName: name,
      isLoading: false,
    );
  }
}
