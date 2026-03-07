import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/features/chat/widgets/message_input.dart';

void main() {
  group('MessageInput', () {
    testWidgets('sends text when enabled', (WidgetTester tester) async {
      String? sent;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MessageInput(
              canSend: true,
              onSend: (value) {
                sent = value;
              },
            ),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), 'hello');
      await tester.pump();
      await tester.tap(find.byIcon(Icons.arrow_upward));
      await tester.pump();

      expect(sent, 'hello');
    });

    testWidgets('keeps send button disabled when cannot send', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: MessageInput(canSend: false, onSend: _noop)),
        ),
      );

      await tester.enterText(find.byType(TextField), 'hello');
      await tester.pump();

      final buttons = tester.widgetList<IconButton>(find.byType(IconButton)).toList();
      final button = buttons.last;
      expect(button.onPressed, isNull);
    });
  });
}

void _noop(String _) {}
