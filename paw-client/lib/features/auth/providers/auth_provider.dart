import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/auth/session_events.dart';
import '../../../core/auth/token_storage.dart';
import '../../../core/di/service_locator.dart';
import '../../../core/errors/app_error.dart';
import '../../../core/http/api_client.dart';
import '../../../core/observability/app_logger.dart';
import '../../../core/ws/ws_service.dart';

enum AuthStep {
  authMethodSelect,
  phoneInput,
  otpVerify,
  deviceName,
  usernameSetup,
  authenticated,
}

class AuthState {
  final AuthStep step;
  final String phone;
  final String deviceName;
  final String username;
  final bool discoverableByPhone;
  final String? sessionToken;
  final String? accessToken;
  final String? refreshToken;
  final bool isLoading;
  final String? error;

  const AuthState({
    this.step = AuthStep.authMethodSelect,
    this.phone = '',
    this.deviceName = '',
    this.username = '',
    this.discoverableByPhone = false,
    this.sessionToken,
    this.accessToken,
    this.refreshToken,
    this.isLoading = false,
    this.error,
  });

  const AuthState.initial()
    : step = AuthStep.authMethodSelect,
      phone = '',
      deviceName = '',
      username = '',
      discoverableByPhone = false,
      sessionToken = null,
      accessToken = null,
      refreshToken = null,
      isLoading = false,
      error = null;

  AuthState copyWith({
    AuthStep? step,
    String? phone,
    String? deviceName,
    String? username,
    bool? discoverableByPhone,
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
      username: username ?? this.username,
      discoverableByPhone: discoverableByPhone ?? this.discoverableByPhone,
      sessionToken: sessionToken == _unset
          ? this.sessionToken
          : sessionToken as String?,
      accessToken: accessToken == _unset
          ? this.accessToken
          : accessToken as String?,
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
  final TokenStorage _tokenStorage = const TokenStorage();
  StreamSubscription<SessionEvent>? _sessionEventSubscription;

  ApiClient? get _apiClient =>
      getIt.isRegistered<ApiClient>() ? getIt<ApiClient>() : null;
  WsService? get _wsService =>
      getIt.isRegistered<WsService>() ? getIt<WsService>() : null;
  SessionEvents? get _sessionEvents =>
      getIt.isRegistered<SessionEvents>() ? getIt<SessionEvents>() : null;

  bool get isAuthenticated => state.step == AuthStep.authenticated;

  String? _queryParam(String key) {
    final direct = Uri.base.queryParameters[key];
    if (direct != null && direct.isNotEmpty) {
      return direct;
    }

    final fragment = Uri.base.fragment;
    final queryIndex = fragment.indexOf('?');
    if (queryIndex == -1 || queryIndex == fragment.length - 1) {
      return null;
    }

    return Uri.splitQueryString(fragment.substring(queryIndex + 1))[key];
  }

  @override
  AuthState build() {
    final queryAccessToken = kIsWeb ? _queryParam('e2e_access_token') : null;
    final queryRefreshToken = kIsWeb ? _queryParam('e2e_refresh_token') : null;

    _sessionEventSubscription ??= _sessionEvents?.stream.listen((event) {
      if (event.reason == SessionExpiryReason.unauthorized) {
        unawaited(_clearSession());
      }
    });
    ref.onDispose(() => _sessionEventSubscription?.cancel());

    if (kIsWeb) {
      unawaited(_bootstrapWebSessionFromQuery());
    } else {
      unawaited(_restoreSession());
    }

    if (queryAccessToken != null && queryAccessToken.isNotEmpty) {
      return AuthState(
        step: AuthStep.authenticated,
        accessToken: queryAccessToken,
        refreshToken: queryRefreshToken,
        isLoading: true,
      );
    }

    return const AuthState.initial();
  }

  void showPhoneOtp() {
    state = state.copyWith(step: AuthStep.phoneInput, error: null);
  }

  void backToAuthMethodSelect() {
    state = state.copyWith(
      step: AuthStep.authMethodSelect,
      phone: '',
      error: null,
    );
  }

  Future<void> _bootstrapWebSessionFromQuery() async {
    final apiClient = _apiClient;
    final wsService = _wsService;
    if (apiClient == null) {
      return;
    }

    final accessToken = _queryParam('e2e_access_token');
    final refreshToken = _queryParam('e2e_refresh_token');
    if (accessToken == null || accessToken.isEmpty) {
      return;
    }

    apiClient.setToken(accessToken);

    try {
      final me = await apiClient.getMe();
      await wsService?.connect(wsService.serverUrl, accessToken);

      AppLogger.event(
        'auth.session.bootstrap.success',
        data: {'platform': 'web'},
      );

      state = state.copyWith(
        step: AuthStep.authenticated,
        accessToken: accessToken,
        refreshToken: refreshToken,
        username: (me['username'] as String?) ?? '',
        discoverableByPhone: me['discoverable_by_phone'] == true,
        isLoading: false,
        error: null,
      );
    } catch (error) {
      final uiError = AppErrorMapper.map(error);
      AppLogger.event(
        'auth.session.bootstrap.failed',
        data: {'code': uiError.code.name, 'detail': uiError.message},
      );
      await _clearSession();
    }
  }

  Future<void> _restoreSession() async {
    final apiClient = _apiClient;
    final wsService = _wsService;
    if (apiClient == null) {
      return;
    }

    final storedTokens = await _tokenStorage.read();
    if (storedTokens == null) {
      return;
    }

    apiClient.setToken(storedTokens.accessToken);

    try {
      final me = await apiClient.getMe();
      await wsService?.connect(wsService.serverUrl, storedTokens.accessToken);

      AppLogger.event(
        'auth.session.restore.success',
        data: {'platform': kIsWeb ? 'web' : 'native'},
      );

      state = state.copyWith(
        step: AuthStep.authenticated,
        accessToken: storedTokens.accessToken,
        refreshToken: storedTokens.refreshToken,
        username: (me['username'] as String?) ?? '',
        discoverableByPhone: me['discoverable_by_phone'] == true,
        error: null,
      );
    } catch (error) {
      final uiError = AppErrorMapper.map(error);
      AppLogger.event(
        'auth.session.restore.failed',
        data: {'code': uiError.code.name, 'detail': uiError.message},
      );
      await _clearSession();
    }
  }

  Future<void> requestOtp(String phone) async {
    state = state.copyWith(isLoading: true, error: null);

    final apiClient = _apiClient;
    if (apiClient == null) {
      state = state.copyWith(
        isLoading: false,
        error: 'ApiClient not configured',
      );
      return;
    }

    try {
      await apiClient.requestOtp(phone);
      AppLogger.event('auth.login.request_otp.success');
      state = state.copyWith(
        step: AuthStep.otpVerify,
        phone: phone,
        isLoading: false,
        error: null,
      );
    } catch (error) {
      final uiError = AppErrorMapper.map(error);
      AppLogger.event(
        'auth.login.request_otp.failed',
        data: {'code': uiError.code.name, 'detail': uiError.message},
      );
      state = state.copyWith(isLoading: false, error: uiError.message);
    }
  }

  Future<void> verifyOtp(String code) async {
    state = state.copyWith(isLoading: true, error: null);

    final apiClient = _apiClient;
    if (apiClient == null) {
      state = state.copyWith(
        isLoading: false,
        error: 'ApiClient not configured',
      );
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
      final uiError = AppErrorMapper.map(error);
      AppLogger.event(
        'auth.login.verify_otp.failed',
        data: {'code': uiError.code.name, 'detail': uiError.message},
      );
      state = state.copyWith(isLoading: false, error: uiError.message);
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

      apiClient.setToken(accessToken);
      final me = await apiClient.getMe();
      if (wsService != null) {
        await wsService.connect(wsService.serverUrl, accessToken);
      }

      await _tokenStorage.write(
        accessToken: accessToken,
        refreshToken: refreshToken,
      );

      AppLogger.event('auth.login.success');

      final username = (me['username'] as String?) ?? '';
      final discoverableByPhone = me['discoverable_by_phone'] == true;

      state = state.copyWith(
        step: username.isEmpty
            ? AuthStep.usernameSetup
            : AuthStep.authenticated,
        deviceName: name,
        accessToken: accessToken,
        refreshToken: refreshToken,
        username: username,
        discoverableByPhone: discoverableByPhone,
        isLoading: false,
        error: null,
      );
    } catch (error) {
      await _tokenStorage.clear();
      apiClient.clearToken();
      await wsService?.disconnect();

      final uiError = AppErrorMapper.map(error);
      AppLogger.event(
        'auth.login.register_device.failed',
        data: {'code': uiError.code.name, 'detail': uiError.message},
      );
      state = state.copyWith(isLoading: false, error: uiError.message);
    }
  }

  Future<void> completeUsernameSetup({
    required String username,
    required bool discoverableByPhone,
  }) async {
    state = state.copyWith(isLoading: true, error: null);

    final apiClient = _apiClient;
    if (apiClient == null || state.accessToken == null) {
      state = state.copyWith(
        isLoading: false,
        error: 'Authentication required',
      );
      return;
    }

    try {
      final updated = await apiClient.updateMe(
        username: username,
        discoverableByPhone: discoverableByPhone,
      );

      state = state.copyWith(
        step: AuthStep.authenticated,
        username: (updated['username'] as String?) ?? username,
        discoverableByPhone: updated['discoverable_by_phone'] == null
            ? discoverableByPhone
            : updated['discoverable_by_phone'] == true,
        isLoading: false,
        error: null,
      );
    } catch (error) {
      final uiError = AppErrorMapper.map(error);
      AppLogger.event(
        'auth.login.username_setup.failed',
        data: {'code': uiError.code.name, 'detail': uiError.message},
      );
      state = state.copyWith(isLoading: false, error: uiError.message);
    }
  }

  void skipUsernameSetup() {
    state = state.copyWith(
      step: AuthStep.authenticated,
      isLoading: false,
      error: null,
    );
  }

  Future<void> logout() async {
    await _clearSession();
  }

  Future<void> _clearSession() async {
    await _tokenStorage.clear();
    _apiClient?.clearToken();
    await _wsService?.disconnect();
    AppLogger.event('auth.session.cleared');

    state = const AuthState.initial();
  }
}
