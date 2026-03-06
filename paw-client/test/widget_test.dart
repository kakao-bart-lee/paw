import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:paw_client/main.dart';
import 'package:paw_client/core/router/app_router.dart';

void main() {
  testWidgets('App renders without crashing', (WidgetTester tester) async {
    // Create a minimal router for testing
    final testRouter = GoRouter(
      routes: [],
    );

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          appRouterProvider.overrideWithValue(testRouter),
        ],
        child: const PawApp(),
      ),
    );
    // MaterialApp.router creates a MaterialApp widget, so this should find it
    expect(find.byType(MaterialApp), findsOneWidget);
  });

  testWidgets('Dark theme is default', (WidgetTester tester) async {
    // Create a minimal router for testing
    final testRouter = GoRouter(
      routes: [],
    );

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          appRouterProvider.overrideWithValue(testRouter),
        ],
        child: const PawApp(),
      ),
    );
    // Find the MaterialApp widget created by MaterialApp.router
    final MaterialApp app = tester.widget<MaterialApp>(find.byType(MaterialApp));
    expect(app.themeMode, ThemeMode.dark);
  });
}
