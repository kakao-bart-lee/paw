import 'dart:io' show Platform;
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/platform/desktop_service.dart';
import 'package:paw_client/features/chat/screens/conversations_screen.dart';

void main() {
  group('DesktopService', () {
    late DesktopService service;

    setUp(() {
      service = DesktopService();
    });

    test('isDesktop returns correct value for current platform', () {
      // Tests run on the host machine, so the result depends on the OS.
      final expected = !kIsWeb &&
          (Platform.isMacOS || Platform.isWindows || Platform.isLinux);
      expect(service.isDesktop, expected);
    });

    test('setupSystemTray executes without error', () {
      // Should not throw regardless of platform.
      expect(() => service.setupSystemTray(), returnsNormally);
    });

    test('registerKeyboardShortcuts executes without error', () {
      // Should not throw regardless of platform.
      expect(() => service.registerKeyboardShortcuts(), returnsNormally);
    });
  });

  group('Responsive breakpoint', () {
    testWidgets('shows single-panel layout on narrow screens',
        (WidgetTester tester) async {
      // Simulate a narrow (mobile) viewport – 375 px wide.
      tester.view.physicalSize = const Size(375, 812);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        const MaterialApp(
          home: ConversationsScreen(),
        ),
      );
      await tester.pump();

      // On narrow screens there should be NO VerticalDivider (single panel).
      expect(find.byType(VerticalDivider), findsNothing);
    });

    testWidgets('shows two-panel layout on wide screens',
        (WidgetTester tester) async {
      // Simulate a wide (desktop) viewport – 1200 px wide.
      tester.view.physicalSize = const Size(1200, 800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        const MaterialApp(
          home: ConversationsScreen(),
        ),
      );
      await tester.pump();

      // On wide screens the VerticalDivider separating the two panels
      // should be present.
      expect(find.byType(VerticalDivider), findsOneWidget);
    });
  });
}
