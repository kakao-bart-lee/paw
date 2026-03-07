import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/features/chat/models/conversation.dart';
import 'package:paw_client/features/chat/providers/chat_provider.dart';
import 'package:paw_client/features/chat/screens/conversations_screen.dart';

void main() {
  group('ConversationsScreen states', () {
    testWidgets('shows loading indicator when loading', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            conversationsLoadStateProvider.overrideWith(
              (ref) => ResourceLoadState.loading,
            ),
            conversationsNotifierProvider.overrideWith(
              () => _EmptyConversationsNotifier(),
            ),
          ],
          child: const MaterialApp(home: ConversationsScreen()),
        ),
      );

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('shows error text when failed', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            conversationsLoadStateProvider.overrideWith(
              (ref) => ResourceLoadState.error,
            ),
            conversationsErrorProvider.overrideWith((ref) => '로드 실패'),
            conversationsNotifierProvider.overrideWith(
              () => _EmptyConversationsNotifier(),
            ),
          ],
          child: const MaterialApp(home: ConversationsScreen()),
        ),
      );

      expect(find.text('로드 실패'), findsOneWidget);
    });

    testWidgets('shows empty state when ready and no conversations', (
      tester,
    ) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            conversationsLoadStateProvider.overrideWith(
              (ref) => ResourceLoadState.ready,
            ),
            conversationsNotifierProvider.overrideWith(
              () => _EmptyConversationsNotifier(),
            ),
          ],
          child: const MaterialApp(home: ConversationsScreen()),
        ),
      );

      expect(find.text('아직 대화가 없습니다'), findsOneWidget);
    });
  });
}

class _EmptyConversationsNotifier extends ConversationsNotifier {
  @override
  List<Conversation> build() => const [];
}
