import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/di/service_locator.dart';
import 'package:paw_client/core/http/api_client.dart';
import 'package:paw_client/core/ws/reconnection_manager.dart';
import 'package:paw_client/core/ws/ws_service.dart';
import 'package:paw_client/features/chat/models/conversation.dart';
import 'package:paw_client/features/chat/models/message.dart';
import 'package:paw_client/features/chat/providers/chat_provider.dart';

void main() {
  group('MessagesNotifier send guards', () {
    tearDown(() async {
      await getIt.reset();
    });

    test('fails when auth token is missing', () async {
      await getIt.reset();
      getIt.registerSingleton<ApiClient>(_FakeApiClient());
      getIt.registerSingleton<WsService>(
        _FakeWsService(isConnectedValue: true),
      );

      final container = ProviderContainer(
        overrides: [
          conversationsNotifierProvider.overrideWith(
            () => _SeedConversationsNotifier(),
          ),
          messagesNotifierProvider(
            'conv_1',
          ).overrideWith(() => _SeedMessagesNotifier('conv_1')),
        ],
      );
      addTearDown(container.dispose);

      final notifier = container.read(
        messagesNotifierProvider('conv_1').notifier,
      );
      final result = await notifier.sendMessage('hello');

      expect(result.ok, isFalse);
      expect(result.message, contains('로그인'));
    });

    test('fails when ws is disconnected', () async {
      await getIt.reset();
      final api = _FakeApiClient();
      api.setToken('token');
      getIt.registerSingleton<ApiClient>(api);
      getIt.registerSingleton<WsService>(
        _FakeWsService(isConnectedValue: false),
      );

      final container = ProviderContainer(
        overrides: [
          conversationsNotifierProvider.overrideWith(
            () => _SeedConversationsNotifier(),
          ),
          messagesNotifierProvider(
            'conv_1',
          ).overrideWith(() => _SeedMessagesNotifier('conv_1')),
        ],
      );
      addTearDown(container.dispose);

      final notifier = container.read(
        messagesNotifierProvider('conv_1').notifier,
      );
      final result = await notifier.sendMessage('hello');

      expect(result.ok, isFalse);
      expect(result.message, contains('실시간 연결'));
    });

    test('rolls back optimistic message on send failure', () async {
      await getIt.reset();
      final api = _FakeApiClient(throwOnSend: true);
      api.setToken('token');
      getIt.registerSingleton<ApiClient>(api);
      getIt.registerSingleton<WsService>(
        _FakeWsService(isConnectedValue: true),
      );

      final container = ProviderContainer(
        overrides: [
          conversationsNotifierProvider.overrideWith(
            () => _SeedConversationsNotifier(),
          ),
          messagesNotifierProvider(
            'conv_1',
          ).overrideWith(() => _SeedMessagesNotifier('conv_1')),
        ],
      );
      addTearDown(container.dispose);

      final notifier = container.read(
        messagesNotifierProvider('conv_1').notifier,
      );
      final result = await notifier.sendMessage('hello');
      final messages = container.read(messagesNotifierProvider('conv_1'));

      expect(result.ok, isFalse);
      expect(messages, isEmpty);
    });
  });
}

class _SeedConversationsNotifier extends ConversationsNotifier {
  @override
  List<Conversation> build() {
    return [
      Conversation(
        id: 'conv_1',
        name: 'seed',
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

class _FakeApiClient extends ApiClient {
  _FakeApiClient({this.throwOnSend = false})
    : super(baseUrl: 'http://localhost:3000');

  final bool throwOnSend;

  @override
  Future<Map<String, dynamic>> sendMessage(
    String convId,
    String content,
    String idempotencyKey,
  ) async {
    if (throwOnSend) {
      throw ApiException.fromStatusCode(503, 'server down');
    }
    return {
      'id': 'srv_1',
      'conversation_id': convId,
      'sender_id': 'me',
      'content': content,
      'format': 'plain',
      'seq': 1,
      'created_at': DateTime.now().toIso8601String(),
    };
  }
}

class _FakeWsService extends WsService {
  _FakeWsService({required this.isConnectedValue})
    : super(
        serverUrl: 'http://localhost:3000',
        reconnectionManager: ReconnectionManager(),
      );

  final bool isConnectedValue;

  @override
  bool get isConnected => isConnectedValue;
}
