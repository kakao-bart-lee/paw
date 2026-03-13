import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/features/settings/screens/settings_screen.dart';

void main() {
  testWidgets('shows web/desktop session messaging instead of biometric lock', (
    tester,
  ) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: MaterialApp(home: SettingsScreen()),
      ),
    );

    expect(find.text('생체 잠금'), findsNothing);
    expect(find.text('설정'), findsOneWidget);
  });
}
