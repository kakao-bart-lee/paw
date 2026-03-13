import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/di/service_locator.dart';
import 'package:paw_client/core/http/api_client.dart';
import 'package:paw_client/core/ws/reconnection_manager.dart';
import 'package:paw_client/core/ws/ws_service.dart';
import 'package:paw_client/features/chat/models/conversation.dart';
import 'package:paw_client/features/chat/models/message.dart';
import 'package:paw_client/features/chat/providers/chat_provider.dart';
import 'package:paw_client/features/chat/screens/chat_screen.dart';

void main() {
  group('ChatScreen states', () {
    setUp(() async {
      await getIt.reset();
    });

    tearDown(() async {
      if (getIt.isRegistered<WsService>()) {
        await getIt<WsService>().dispose();
      }
      await getIt.reset();
    });

    testWidgets('shows loading indicator when messages are loading', (
      tester,
    ) async {
      getIt.registerSingleton<ApiClient>(
        ApiClient(baseUrl: 'http://localhost:38173'),
      );
      getIt.registerSingleton<WsService>(
        _FakeWsService(state: WsConnectionState.connected, hasToken: true),
      );

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
      getIt.registerSingleton<ApiClient>(
        ApiClient(baseUrl: 'http://localhost:38173'),
      );
      getIt.registerSingleton<WsService>(
        _FakeWsService(state: WsConnectionState.connected, hasToken: true),
      );

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
      getIt.registerSingleton<ApiClient>(
        ApiClient(baseUrl: 'http://localhost:38173'),
      );
      getIt.registerSingleton<WsService>(
        _FakeWsService(state: WsConnectionState.connected, hasToken: true),
      );

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

    testWidgets('shows reconnecting guidance when ws is retrying', (
      tester,
    ) async {
      final apiClient = ApiClient(baseUrl: 'http://localhost:38173');
      apiClient.setToken('token');
      getIt.registerSingleton<ApiClient>(apiClient);
      getIt.registerSingleton<WsService>(
        _FakeWsService(state: WsConnectionState.retrying, hasToken: true),
      );

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

      expect(find.text('연결이 끊겨 재시도 중입니다...'), findsOneWidget);
      await tester.enterText(
        find.byKey(const ValueKey('chat-message-input')),
        'hello',
      );
      await tester.pump();

      final sendButton = tester.widget<IconButton>(
        find.byKey(const ValueKey('chat-send-button')),
      );
      expect(sendButton.onPressed, isNull);
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

class _FakeWsService extends WsService {
  _FakeWsService({required WsConnectionState state, required bool hasToken})
    : _state = ValueNotifier(state),
      super(
        serverUrl: 'http://localhost:38173',
        reconnectionManager: ReconnectionManager(),
      );

  final ValueNotifier<WsConnectionState> _state;

  @override
  bool get isConnected => _state.value == WsConnectionState.connected;

  @override
  ValueListenable<WsConnectionState> get connectionState => _state;

  @override
  Future<void> dispose() async {
    _state.dispose();
  }
}
