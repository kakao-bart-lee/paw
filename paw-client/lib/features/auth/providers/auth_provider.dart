import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

import '../../../core/di/service_locator.dart';
import '../../../core/http/api_client.dart';
import '../../../core/ws/ws_service.dart';

enum AuthStep { phoneInput, otpVerify, deviceName, authenticated }

class AuthState {
  final AuthStep step;
  final String phone;
  final String deviceName;
  final String? sessionToken;
  final String? accessToken;
  final String? refreshToken;
  final bool isLoading;
  final String? error;

  const AuthState({
    this.step = AuthStep.phoneInput,
    this.phone = '',
    this.deviceName = '',
    this.sessionToken,
    this.accessToken,
    this.refreshToken,
    this.isLoading = false,
    this.error,
  });

  const AuthState.initial()
      : step = AuthStep.phoneInput,
        phone = '',
        deviceName = '',
        sessionToken = null,
        accessToken = null,
        refreshToken = null,
        isLoading = false,
        error = null;

  AuthState copyWith({
    AuthStep? step,
    String? phone,
    String? deviceName,
    Object? sessionToken = _unset,
    Object? accessToken = _unset,
    Object? refreshToken = _unset,
    bool? isLoading,
    Object? error = _unset,
  }) {
    return AuthState(
      step: step ?? this.step,
      phone: phone ?? this.phone,
      deviceName: deviceName ?? this.deviceName,
      sessionToken: sessionToken == _unset
          ? this.sessionToken
          : sessionToken as String?,
      accessToken:
          accessToken == _unset ? this.accessToken : accessToken as String?,
      refreshToken: refreshToken == _unset
          ? this.refreshToken
          : refreshToken as String?,
      isLoading: isLoading ?? this.isLoading,
      error: error == _unset ? this.error : error as String?,
    );
  }
}

const _unset = Object();

final authNotifierProvider = NotifierProvider<AuthNotifier, AuthState>(
  AuthNotifier.new,
);

class AuthNotifier extends Notifier<AuthState> {
  static const _secureStorage = FlutterSecureStorage();
  static const _accessTokenKey = 'access_token';
  static const _refreshTokenKey = 'refresh_token';

  ApiClient? get _apiClient =>
      getIt.isRegistered<ApiClient>() ? getIt<ApiClient>() : null;
  WsService? get _wsService =>
      getIt.isRegistered<WsService>() ? getIt<WsService>() : null;

  @override
  AuthState build() {
    unawaited(_restoreSession());
    return const AuthState.initial();
  }

  Future<void> _restoreSession() async {
    final apiClient = _apiClient;
    final wsService = _wsService;
    if (apiClient == null) {
      return;
    }

    final accessToken = await _secureStorage.read(key: _accessTokenKey);
    final refreshToken = await _secureStorage.read(key: _refreshTokenKey);
    if (accessToken == null || refreshToken == null) {
      return;
    }

    apiClient.setToken(accessToken);

    try {
      await apiClient.getMe();
      await wsService?.connect(wsService.serverUrl, accessToken);

      state = state.copyWith(
        step: AuthStep.authenticated,
        accessToken: accessToken,
        refreshToken: refreshToken,
        error: null,
      );
    } catch (_) {
      await _secureStorage.delete(key: _accessTokenKey);
      await _secureStorage.delete(key: _refreshTokenKey);
      apiClient.clearToken();

      state = state.copyWith(
        step: AuthStep.phoneInput,
        accessToken: null,
        refreshToken: null,
        error: null,
      );
    }
  }

  Future<void> requestOtp(String phone) async {
    state = state.copyWith(isLoading: true, error: null);

    final apiClient = _apiClient;
    if (apiClient == null) {
      state = state.copyWith(isLoading: false, error: 'ApiClient not configured');
      return;
    }

    try {
      await apiClient.requestOtp(phone);
      state = state.copyWith(
        step: AuthStep.otpVerify,
        phone: phone,
        isLoading: false,
        error: null,
      );
    } catch (error) {
      state = state.copyWith(isLoading: false, error: error.toString());
    }
  }

  Future<void> verifyOtp(String code) async {
    state = state.copyWith(isLoading: true, error: null);

    final apiClient = _apiClient;
    if (apiClient == null) {
      state = state.copyWith(isLoading: false, error: 'ApiClient not configured');
      return;
    }

    try {
      final result = await apiClient.verifyOtp(state.phone, code);
      final sessionToken = result['session_token'] as String?;
      if (sessionToken == null || sessionToken.isEmpty) {
        throw Exception('Missing session token');
      }

      state = state.copyWith(
        step: AuthStep.deviceName,
        sessionToken: sessionToken,
        isLoading: false,
        error: null,
      );
    } catch (error) {
      state = state.copyWith(isLoading: false, error: error.toString());
    }
  }

  Future<void> setDeviceName(String name) async {
    state = state.copyWith(isLoading: true, error: null);

    final apiClient = _apiClient;
    final wsService = _wsService;
    final sessionToken = state.sessionToken;
    if (apiClient == null || sessionToken == null || sessionToken.isEmpty) {
      state = state.copyWith(
        isLoading: false,
        error: 'Missing session token for device registration',
      );
      return;
    }

    try {
      // Ed25519 generation is handled later (T23). Stub format for now.
      const stubEd25519PubKeyBase64 =
          'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=';
      final result = await apiClient.registerDevice(
        sessionToken,
        name,
        stubEd25519PubKeyBase64,
      );

      final accessToken = result['access_token'] as String?;
      final refreshToken = result['refresh_token'] as String?;
      if (accessToken == null || refreshToken == null) {
        throw Exception('Missing tokens from register-device response');
      }

      await _secureStorage.write(key: _accessTokenKey, value: accessToken);
      await _secureStorage.write(key: _refreshTokenKey, value: refreshToken);

      apiClient.setToken(accessToken);
      if (wsService != null) {
        await wsService.connect(wsService.serverUrl, accessToken);
      }

      state = state.copyWith(
        step: AuthStep.authenticated,
        deviceName: name,
        accessToken: accessToken,
        refreshToken: refreshToken,
        isLoading: false,
        error: null,
      );
    } catch (error) {
      state = state.copyWith(isLoading: false, error: error.toString());
    }
  }
}
