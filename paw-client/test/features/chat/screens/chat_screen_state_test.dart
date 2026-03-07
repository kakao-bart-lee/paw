import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/features/chat/models/conversation.dart';
import 'package:paw_client/features/chat/models/message.dart';
import 'package:paw_client/features/chat/providers/chat_provider.dart';
import 'package:paw_client/features/chat/screens/chat_screen.dart';

void main() {
  group('ChatScreen states', () {
    testWidgets('shows loading indicator when messages are loading', (
      tester,
    ) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            conversationsNotifierProvider.overrideWith(
              () => _SeedConversationsNotifier(),
            ),
            messagesNotifierProvider(
              'conv_1',
            ).overrideWith(() => _SeedMessagesNotifier('conv_1')),
            messagesLoadStateProvider(
              'conv_1',
            ).overrideWith((ref) => ResourceLoadState.loading),
          ],
          child: const MaterialApp(home: ChatScreen(conversationId: 'conv_1')),
        ),
      );

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('shows error text when messages fail to load', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            conversationsNotifierProvider.overrideWith(
              () => _SeedConversationsNotifier(),
            ),
            messagesNotifierProvider(
              'conv_1',
            ).overrideWith(() => _SeedMessagesNotifier('conv_1')),
            messagesLoadStateProvider(
              'conv_1',
            ).overrideWith((ref) => ResourceLoadState.error),
            messagesErrorProvider('conv_1').overrideWith((ref) => '메시지 로드 실패'),
          ],
          child: const MaterialApp(home: ChatScreen(conversationId: 'conv_1')),
        ),
      );

      expect(find.text('메시지 로드 실패'), findsOneWidget);
    });

    testWidgets('shows empty text when ready without messages', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            conversationsNotifierProvider.overrideWith(
              () => _SeedConversationsNotifier(),
            ),
            messagesNotifierProvider(
              'conv_1',
            ).overrideWith(() => _SeedMessagesNotifier('conv_1')),
            messagesLoadStateProvider(
              'conv_1',
            ).overrideWith((ref) => ResourceLoadState.ready),
          ],
          child: const MaterialApp(home: ChatScreen(conversationId: 'conv_1')),
        ),
      );

      expect(find.text('메시지가 없습니다.'), findsOneWidget);
    });
  });
}

class _SeedConversationsNotifier extends ConversationsNotifier {
  @override
  List<Conversation> build() {
    return [
      Conversation(
        id: 'conv_1',
        name: 'test',
        unreadCount: 0,
        updatedAt: DateTime.now(),
      ),
    ];
  }
}

class _SeedMessagesNotifier extends MessagesNotifier {
  _SeedMessagesNotifier(super.conversationId);

  @override
  List<Message> build() => const [];
}
