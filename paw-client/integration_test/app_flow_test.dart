import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:integration_test/integration_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:paw_client/core/router/app_router.dart';
import 'package:paw_client/features/auth/providers/auth_provider.dart';
import 'package:paw_client/features/chat/models/conversation.dart';
import 'package:paw_client/features/chat/models/message.dart';
import 'package:paw_client/features/chat/providers/chat_provider.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await initializeDateFormatting('ko_KR');
  });

  group('integration flow', () {
    testWidgets('unauthenticated user is routed to login flow', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            authNotifierProvider.overrideWith(
              () => _UnauthenticatedAuthNotifier(),
            ),
          ],
          child: const _RouterHarness(),
        ),
      );

      await tester.pump();
      await tester.pump(const Duration(milliseconds: 250));

      expect(find.text('Paw'), findsOneWidget);
      expect(find.text('AI-Native Messenger'), findsOneWidget);
    });

    testWidgets('authenticated flow: chat open + rebuild restore', (
      tester,
    ) async {
      const conversationName = '테스트 대화방';

      final app = ProviderScope(
        overrides: [
          authNotifierProvider.overrideWith(() => _AuthenticatedAuthNotifier()),
          conversationsNotifierProvider.overrideWith(
            () => _SeedConversationsNotifier(),
          ),
          messagesNotifierProvider(
            'conv_1',
          ).overrideWith(() => _SeedMessagesNotifier('conv_1')),
          conversationsLoadStateProvider.overrideWith(
            (ref) => ResourceLoadState.ready,
          ),
          conversationsErrorProvider.overrideWith((ref) => null),
          messagesLoadStateProvider(
            'conv_1',
          ).overrideWith((ref) => ResourceLoadState.ready),
          messagesErrorProvider('conv_1').overrideWith((ref) => null),
        ],
        child: const _RouterHarness(),
      );

      await tester.pumpWidget(app);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));

      expect(find.text(conversationName), findsAtLeastNWidgets(1));

      await tester.tap(find.text(conversationName).first);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));

      expect(find.byType(TextField), findsOneWidget);
      expect(find.text('메시지가 없습니다.'), findsOneWidget);

      // Simulate restart/rebuild.
      await tester.pumpWidget(app);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));

      expect(find.text(conversationName), findsAtLeastNWidgets(1));
    });
  });
}

class _RouterHarness extends ConsumerWidget {
  const _RouterHarness();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(appRouterProvider);
    return MaterialApp.router(routerConfig: router);
  }
}

class _AuthenticatedAuthNotifier extends AuthNotifier {
  @override
  AuthState build() {
    return const AuthState(
      step: AuthStep.authenticated,
      accessToken: 'test-access-token',
      refreshToken: 'test-refresh-token',
    );
  }
}

class _UnauthenticatedAuthNotifier extends AuthNotifier {
  @override
  AuthState build() {
    return const AuthState.initial();
  }
}

class _SeedConversationsNotifier extends ConversationsNotifier {
  @override
  List<Conversation> build() {
    return [
      Conversation(
        id: 'conv_1',
        name: '테스트 대화방',
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
