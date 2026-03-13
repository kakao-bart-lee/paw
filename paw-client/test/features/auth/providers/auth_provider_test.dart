import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
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
      getIt.registerSingleton<WsService>(
        WsService(
          serverUrl: 'http://localhost:38173',
          reconnectionManager: ReconnectionManager(),
        ),
      );
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
      expect(state.step, AuthStep.phoneInput);
      expect(state.accessToken, isNull);
      expect(state.refreshToken, isNull);
      expect(state.sessionToken, isNull);
    });
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
