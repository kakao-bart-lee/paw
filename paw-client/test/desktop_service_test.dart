import 'dart:io' show Platform;
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/platform/desktop_service.dart';
import 'package:paw_client/features/chat/screens/conversations_screen.dart';
import 'package:paw_client/features/chat/providers/chat_provider.dart';
import 'package:paw_client/features/chat/models/conversation.dart';

void main() {
  group('DesktopService', () {
    late DesktopService service;

    setUp(() {
      service = DesktopService();
    });

    test('isDesktop returns correct value for current platform', () {
      final expected = !kIsWeb &&
          (Platform.isMacOS || Platform.isWindows || Platform.isLinux);
      expect(service.isDesktop, expected);
    });

    test('setupSystemTray executes without error', () {
      expect(() => service.setupSystemTray(), returnsNormally);
    });

    test('registerKeyboardShortcuts executes without error', () {
      expect(() => service.registerKeyboardShortcuts(), returnsNormally);
    });
  });

  group('Responsive breakpoint', () {
    /// Build [ConversationsScreen] in a test-safe widget tree.
    ///
    /// The real provider loads mock conversations whose tiles use
    /// [DateFormat] with a Korean locale that isn't initialised in
    /// tests.  We override the provider with an empty list so we can
    /// test layout logic without triggering locale errors.
    Widget buildTestWidget() {
      return ProviderScope(
        overrides: [
          conversationsNotifierProvider.overrideWith(
            () => _EmptyConversationsNotifier(),
          ),
        ],
        child: const MaterialApp(
          home: ConversationsScreen(),
        ),
      );
    }

    testWidgets('shows single-panel layout on narrow screens',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(375, 812);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(buildTestWidget());
      await tester.pump();

      // Single panel → no VerticalDivider.
      expect(find.byType(VerticalDivider), findsNothing);
    });

    testWidgets('shows two-panel layout on wide screens',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(buildTestWidget());
      await tester.pump();

      // Two-panel → VerticalDivider present.
      expect(find.byType(VerticalDivider), findsOneWidget);
    });
  });
}

/// A trivial [ConversationsNotifier] that always returns an empty list.
class _EmptyConversationsNotifier extends ConversationsNotifier {
  @override
  List<Conversation> build() => [];
}
