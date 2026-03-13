import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:paw_client/core/auth/token_storage.dart';
import 'package:paw_client/core/auth/session_events.dart';
import 'package:paw_client/core/di/service_locator.dart';
import 'package:paw_client/core/http/api_client.dart';
import 'package:paw_client/core/ws/reconnection_manager.dart';
import 'package:paw_client/core/ws/ws_service.dart';
import 'package:paw_client/features/auth/providers/auth_provider.dart';

void main() {
  group('AuthNotifier', () {
    setUp(() async {
      await getIt.reset();
      getIt.registerSingleton<SessionEvents>(SessionEvents());
      getIt.registerSingleton<ApiClient>(
        ApiClient(baseUrl: 'http://localhost:38173'),
      );
      getIt.registerSingleton<WsService>(_FakeWsService());
    });

    tearDown(() async {
      if (getIt.isRegistered<SessionEvents>()) {
        await getIt<SessionEvents>().dispose();
      }
      if (getIt.isRegistered<WsService>()) {
        await getIt<WsService>().dispose();
      }
      await getIt.reset();
    });

    test('clears auth state on unauthorized session event', () async {
      final container = ProviderContainer(
        overrides: [
          authNotifierProvider.overrideWith(() => _AuthenticatedAuthNotifier()),
        ],
      );
      addTearDown(container.dispose);

      expect(container.read(authNotifierProvider).step, AuthStep.authenticated);

      getIt<SessionEvents>().emitUnauthorized();
      await Future<void>.delayed(Duration.zero);
      await Future<void>.delayed(Duration.zero);

      final state = container.read(authNotifierProvider);
      expect(state.step, AuthStep.authMethodSelect);
      expect(state.accessToken, isNull);
      expect(state.refreshToken, isNull);
      expect(state.sessionToken, isNull);
    });

    test(
      'moves to otpVerify and stores requested phone on OTP request',
      () async {
        await getIt.reset();
        final api = _FakeApiClient();
        getIt.registerSingleton<SessionEvents>(SessionEvents());
        getIt.registerSingleton<ApiClient>(api);
        getIt.registerSingleton<WsService>(_FakeWsService());

        final container = ProviderContainer();
        addTearDown(container.dispose);

        const phone = '+821012345678';
        await container.read(authNotifierProvider.notifier).requestOtp(phone);

        expect(api.requestedPhones, [phone]);
        expect(container.read(authNotifierProvider).step, AuthStep.otpVerify);
        expect(container.read(authNotifierProvider).phone, phone);
        expect(container.read(authNotifierProvider).error, isNull);
      },
    );

    test(
      'keeps authMethodSelect step and surfaces OTP request errors',
      () async {
        await getIt.reset();
        getIt.registerSingleton<SessionEvents>(SessionEvents());
        getIt.registerSingleton<ApiClient>(
          _FakeApiClient(
            requestOtpError: ApiException.fromStatusCode(400, 'bad phone'),
          ),
        );
        getIt.registerSingleton<WsService>(_FakeWsService());

        final container = ProviderContainer();
        addTearDown(container.dispose);
        container.read(authNotifierProvider.notifier).showPhoneOtp();

        await container
            .read(authNotifierProvider.notifier)
            .requestOtp('invalid');

        final state = container.read(authNotifierProvider);
        expect(state.step, AuthStep.phoneInput);
        expect(state.error, 'bad phone');
      },
    );

    test(
      'moves to deviceName when OTP verification returns session token',
      () async {
        await getIt.reset();
        getIt.registerSingleton<SessionEvents>(SessionEvents());
        getIt.registerSingleton<ApiClient>(
          _FakeApiClient(sessionToken: 'session-token'),
        );
        getIt.registerSingleton<WsService>(_FakeWsService());

        final container = ProviderContainer(
          overrides: [
            authNotifierProvider.overrideWith(() => _PhoneOtpAuthNotifier()),
          ],
        );
        addTearDown(container.dispose);

        await container.read(authNotifierProvider.notifier).verifyOtp('123456');

        final state = container.read(authNotifierProvider);
        expect(state.step, AuthStep.deviceName);
        expect(state.sessionToken, 'session-token');
        expect(state.error, isNull);
      },
    );

    test('keeps otpVerify step and surfaces verification errors', () async {
      await getIt.reset();
      getIt.registerSingleton<SessionEvents>(SessionEvents());
      getIt.registerSingleton<ApiClient>(
        _FakeApiClient(
          verifyOtpError: ApiException.fromStatusCode(503, 'otp failed'),
        ),
      );
      getIt.registerSingleton<WsService>(_FakeWsService());

      final container = ProviderContainer(
        overrides: [
          authNotifierProvider.overrideWith(() => _PhoneOtpAuthNotifier()),
        ],
      );
      addTearDown(container.dispose);

      await container.read(authNotifierProvider.notifier).verifyOtp('123456');

      final state = container.read(authNotifierProvider);
      expect(state.step, AuthStep.otpVerify);
      expect(state.sessionToken, isNull);
      expect(state.error, '서버에 일시적인 문제가 있습니다. 잠시 후 다시 시도해주세요.');
    });

    test('moves to usernameSetup after device registration', () async {
      await getIt.reset();
      final api = _FakeApiClient();
      getIt.registerSingleton<SessionEvents>(SessionEvents());
      getIt.registerSingleton<ApiClient>(api);
      getIt.registerSingleton<WsService>(_FakeWsService());

      final container = ProviderContainer(
        overrides: [
          authNotifierProvider.overrideWith(() => _DeviceNameAuthNotifier()),
        ],
      );
      addTearDown(container.dispose);

      await container
          .read(authNotifierProvider.notifier)
          .setDeviceName('My Device');

      final state = container.read(authNotifierProvider);
      expect(state.step, AuthStep.usernameSetup);
      expect(state.accessToken, 'access-token');
      expect(state.refreshToken, 'refresh-token');
    });

    test(
      'clears tokens and keeps deviceName step when getMe fails after device registration',
      () async {
        await getIt.reset();
        final api = _FakeApiClient(
          getMeError: ApiException.fromStatusCode(503, 'profile fetch failed'),
        );
        getIt.registerSingleton<SessionEvents>(SessionEvents());
        getIt.registerSingleton<ApiClient>(api);
        getIt.registerSingleton<WsService>(_FakeWsService());

        final storage = const TokenStorage();
        await storage.clear();

        final container = ProviderContainer(
          overrides: [
            authNotifierProvider.overrideWith(() => _DeviceNameAuthNotifier()),
          ],
        );
        addTearDown(() async {
          container.dispose();
          await storage.clear();
        });

        await container
            .read(authNotifierProvider.notifier)
            .setDeviceName('My Device');

        final state = container.read(authNotifierProvider);
        final storedTokens = await storage.read();
        expect(state.step, AuthStep.deviceName);
        expect(state.accessToken, isNull);
        expect(state.refreshToken, isNull);
        expect(state.error, '서버에 일시적인 문제가 있습니다. 잠시 후 다시 시도해주세요.');
        expect(storedTokens, isNull);
      },
    );

    test('can skip username setup and finish auth', () async {
      final container = ProviderContainer(
        overrides: [
          authNotifierProvider.overrideWith(() => _UsernameSetupAuthNotifier()),
        ],
      );
      addTearDown(container.dispose);

      container.read(authNotifierProvider.notifier).skipUsernameSetup();

      expect(container.read(authNotifierProvider).step, AuthStep.authenticated);
    });

    test(
      'finishes auth immediately when registered device already has username',
      () async {
        await getIt.reset();
        final api = _FakeApiClient(
          meResponse: const {
            'username': 'paw_friend',
            'discoverable_by_phone': true,
          },
        );
        getIt.registerSingleton<SessionEvents>(SessionEvents());
        getIt.registerSingleton<ApiClient>(api);
        getIt.registerSingleton<WsService>(_FakeWsService());

        final container = ProviderContainer(
          overrides: [
            authNotifierProvider.overrideWith(() => _DeviceNameAuthNotifier()),
          ],
        );
        addTearDown(container.dispose);

        await container
            .read(authNotifierProvider.notifier)
            .setDeviceName('My Device');

        final state = container.read(authNotifierProvider);
        expect(state.step, AuthStep.authenticated);
        expect(state.username, 'paw_friend');
        expect(state.discoverableByPhone, isTrue);
      },
    );
  });
}

class _AuthenticatedAuthNotifier extends AuthNotifier {
  @override
  AuthState build() {
    super.build();
    return const AuthState(
      step: AuthStep.authenticated,
      accessToken: 'access',
      refreshToken: 'refresh',
      sessionToken: 'session',
    );
  }
}

class _PhoneOtpAuthNotifier extends AuthNotifier {
  @override
  AuthState build() {
    super.build();
    return const AuthState(step: AuthStep.otpVerify, phone: '+821012345678');
  }
}

class _DeviceNameAuthNotifier extends AuthNotifier {
  @override
  AuthState build() {
    super.build();
    return const AuthState(
      step: AuthStep.deviceName,
      phone: '+821012345678',
      sessionToken: 'session-token',
    );
  }
}

class _UsernameSetupAuthNotifier extends AuthNotifier {
  @override
  AuthState build() {
    super.build();
    return const AuthState(
      step: AuthStep.usernameSetup,
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
    );
  }
}

class _FakeApiClient extends ApiClient {
  _FakeApiClient({
    this.requestOtpError,
    this.verifyOtpError,
    this.sessionToken = 'session-token',
    this.meResponse = const <String, dynamic>{},
    this.getMeError,
  }) : super(baseUrl: 'http://localhost:38173');

  final Object? requestOtpError;
  final Object? verifyOtpError;
  final Object? getMeError;
  final String sessionToken;
  final Map<String, dynamic> meResponse;
  final List<String> requestedPhones = [];
  final List<({String phone, String code})> verifiedOtps = [];

  @override
  Future<Map<String, dynamic>> requestOtp(String phone) async {
    requestedPhones.add(phone);
    if (requestOtpError != null) {
      throw requestOtpError!;
    }
    return {'ok': true};
  }

  @override
  Future<Map<String, dynamic>> verifyOtp(String phone, String code) async {
    verifiedOtps.add((phone: phone, code: code));
    if (verifyOtpError != null) {
      throw verifyOtpError!;
    }
    return {'session_token': sessionToken};
  }

  @override
  Future<Map<String, dynamic>> registerDevice(
    String sessionToken,
    String deviceName,
    String ed25519PubKeyBase64,
  ) async {
    return {'access_token': 'access-token', 'refresh_token': 'refresh-token'};
  }

  @override
  Future<Map<String, dynamic>> getMe() async {
    if (getMeError != null) {
      throw getMeError!;
    }
    return meResponse;
  }
}

class _FakeWsService extends WsService {
  _FakeWsService()
    : super(
        serverUrl: 'http://localhost:38173',
        reconnectionManager: ReconnectionManager(),
      );

  @override
  Future<void> connect(String serverUrl, String token) async {}

  @override
  Future<void> disconnect() async {}
}
