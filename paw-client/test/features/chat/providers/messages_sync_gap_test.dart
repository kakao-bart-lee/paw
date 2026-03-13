import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/di/service_locator.dart';
import 'package:paw_client/core/proto/messages.dart';
import 'package:paw_client/core/ws/reconnection_manager.dart';
import 'package:paw_client/core/ws/ws_service.dart';
import 'package:paw_client/features/chat/models/conversation.dart';
import 'package:paw_client/features/chat/models/message.dart';
import 'package:paw_client/features/chat/providers/chat_provider.dart';

void main() {
  group('MessagesNotifier sync/gap rules', () {
    setUp(() async {
      await getIt.reset();
      getIt.registerSingleton<WsService>(_FakeWsService());
    });

    tearDown(() async {
      await getIt.reset();
    });

    test('duplicate seq is ignored and acked with current seq', () {
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
      final fakeWs = getIt<WsService>() as _FakeWsService;

      notifier.addMessageFromWs(_msg(seq: 1));
      notifier.addMessageFromWs(_msg(seq: 1));

      final messages = container.read(messagesNotifierProvider('conv_1'));
      expect(messages.length, 1);
      expect(messages.first.seq, 1);
      expect(fakeWs.lastAckConversationId, 'conv_1');
      expect(fakeWs.lastAckSeq, 1);
    });

    test(
      'gap seq triggers requestSync and does not append out-of-order message',
      () {
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
        final fakeWs = getIt<WsService>() as _FakeWsService;

        notifier.addMessageFromWs(_msg(seq: 1));
        notifier.addMessageFromWs(_msg(seq: 3));

        final messages = container.read(messagesNotifierProvider('conv_1'));
        expect(messages.length, 1);
        expect(messages.first.seq, 1);
        expect(fakeWs.lastSyncConversationId, 'conv_1');
        expect(fakeWs.lastSyncSeq, 1);
      },
    );

    test('contiguous seq appends normally', () {
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

      notifier.addMessageFromWs(_msg(seq: 1));
      notifier.addMessageFromWs(_msg(seq: 2));

      final messages = container.read(messagesNotifierProvider('conv_1'));
      expect(messages.map((m) => m.seq), [1, 2]);
    });
  });
}

MessageReceivedMsg _msg({required int seq}) {
  return MessageReceivedMsg(
    v: 1,
    id: 'm_$seq',
    conversationId: 'conv_1',
    senderId: 'user_2',
    content: 'msg $seq',
    format: 'plain',
    seq: seq,
    createdAt: DateTime.now(),
  );
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

class _FakeWsService extends WsService {
  _FakeWsService()
    : super(
        serverUrl: 'http://localhost:38173',
        reconnectionManager: ReconnectionManager(),
      );

  String? lastSyncConversationId;
  int? lastSyncSeq;
  String? lastAckConversationId;
  int? lastAckSeq;

  @override
  void requestSync(String conversationId, int lastSeq) {
    lastSyncConversationId = conversationId;
    lastSyncSeq = lastSeq;
  }

  @override
  void sendAck(String conversationId, int lastSeq) {
    lastAckConversationId = conversationId;
    lastAckSeq = lastSeq;
  }
}
