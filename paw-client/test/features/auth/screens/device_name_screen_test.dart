import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:paw_client/features/auth/providers/auth_provider.dart';
import 'package:paw_client/features/auth/screens/device_name_screen.dart';
import 'package:paw_client/features/auth/screens/username_setup_screen.dart';

void main() {
  testWidgets('navigates to chat when device registration authenticates user', (
    tester,
  ) async {
    final container = ProviderContainer(
      overrides: [
        authNotifierProvider.overrideWith(
          () => _DeviceNameToAuthenticatedNotifier(),
        ),
      ],
    );
    addTearDown(container.dispose);

    final router = GoRouter(
      initialLocation: '/auth/device-name',
      routes: [
        GoRoute(
          path: '/auth/device-name',
          builder: (context, state) => const DeviceNameScreen(),
        ),
        GoRoute(
          path: '/chat',
          builder: (context, state) => const Scaffold(body: Text('chat-home')),
        ),
      ],
    );

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp.router(routerConfig: router),
      ),
    );

    await tester.tap(find.text('시작하기'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 200));

    expect(find.text('chat-home'), findsOneWidget);
  });

  testWidgets(
    'navigates to username setup when username onboarding is required',
    (tester) async {
      final container = ProviderContainer(
        overrides: [
          authNotifierProvider.overrideWith(
            () => _DeviceNameToUsernameSetupNotifier(),
          ),
        ],
      );
      addTearDown(container.dispose);

      final router = GoRouter(
        initialLocation: '/auth/device-name',
        routes: [
          GoRoute(
            path: '/auth/device-name',
            builder: (context, state) => const DeviceNameScreen(),
          ),
          GoRoute(
            path: '/auth/username-setup',
            builder: (context, state) => const UsernameSetupScreen(),
          ),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp.router(routerConfig: router),
        ),
      );

      await tester.tap(find.text('시작하기'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));

      expect(find.text('username 만들기'), findsOneWidget);
    },
  );
}

class _DeviceNameToAuthenticatedNotifier extends AuthNotifier {
  @override
  AuthState build() {
    return const AuthState(
      step: AuthStep.deviceName,
      sessionToken: 'session-token',
    );
  }

  @override
  Future<void> setDeviceName(String name) async {
    state = state.copyWith(
      step: AuthStep.authenticated,
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
    );
  }
}

class _DeviceNameToUsernameSetupNotifier extends AuthNotifier {
  @override
  AuthState build() {
    return const AuthState(
      step: AuthStep.deviceName,
      sessionToken: 'session-token',
    );
  }

  @override
  Future<void> setDeviceName(String name) async {
    state = state.copyWith(
      step: AuthStep.usernameSetup,
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
    );
  }
}
