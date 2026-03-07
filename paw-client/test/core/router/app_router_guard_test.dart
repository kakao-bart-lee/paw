import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/router/app_router.dart';
import 'package:paw_client/features/auth/providers/auth_provider.dart';

void main() {
  group('App router auth guard', () {
    testWidgets('redirects unauthenticated access to protected route', (
      WidgetTester tester,
    ) async {
      final container = ProviderContainer(
        overrides: [
          authNotifierProvider.overrideWith(
            () => _UnauthenticatedAuthNotifier(),
          ),
        ],
      );
      addTearDown(container.dispose);

      final router = container.read(appRouterProvider);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp.router(routerConfig: router),
        ),
      );

      router.go('/profile/me');
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));

      expect(find.text('Paw'), findsOneWidget);
      expect(find.text('AI-Native Messenger'), findsOneWidget);
    });

    testWidgets('redirects authenticated access from /login to /chat', (
      WidgetTester tester,
    ) async {
      final container = ProviderContainer(
        overrides: [
          authNotifierProvider.overrideWith(() => _AuthenticatedAuthNotifier()),
        ],
      );
      addTearDown(container.dispose);

      final router = container.read(appRouterProvider);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp.router(routerConfig: router),
        ),
      );

      router.go('/login');
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));

      expect(find.text('채팅'), findsAtLeastNWidgets(1));
    });
  });
}

class _AuthenticatedAuthNotifier extends AuthNotifier {
  @override
  AuthState build() {
    return const AuthState(
      step: AuthStep.authenticated,
      accessToken: 'token',
      refreshToken: 'refresh',
    );
  }
}

class _UnauthenticatedAuthNotifier extends AuthNotifier {
  @override
  AuthState build() {
    return const AuthState.initial();
  }
}
