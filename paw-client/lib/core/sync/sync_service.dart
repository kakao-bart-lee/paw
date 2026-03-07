import 'package:drift/drift.dart';

import '../db/app_database.dart';
import '../db/daos/conversations_dao.dart';
import '../db/daos/messages_dao.dart';
import '../observability/app_logger.dart';
import '../proto/messages.dart';

typedef SyncRequestFn = void Function(String conversationId, int lastSeq);

class SyncService {
  final MessagesDao _messagesDao;
  final ConversationsDao _conversationsDao;
  final SyncRequestFn _requestSync;

  SyncService({
    required MessagesDao messagesDao,
    required ConversationsDao conversationsDao,
    required SyncRequestFn requestSync,
  }) : _messagesDao = messagesDao,
       _conversationsDao = conversationsDao,
       _requestSync = requestSync;

  Future<void> syncAllConversations() async {
    AppLogger.event('sync.start');
    final conversations = await _conversationsDao.getAllConversations();

    for (final conversation in conversations) {
      final lastSeq = await _messagesDao.getLastSeq(conversation.id);
      _requestSync(conversation.id, lastSeq);
    }
    AppLogger.event(
      'sync.complete',
      data: {'conversation_count': conversations.length},
    );
  }

  Future<void> persistMessage(MessageReceivedMsg msg) async {
    await _messagesDao.upsertMessage(
      MessagesTableCompanion(
        id: Value(msg.id),
        conversationId: Value(msg.conversationId),
        senderId: Value(msg.senderId),
        content: Value(msg.content),
        format: Value(msg.format),
        seq: Value(msg.seq),
        createdAt: Value(msg.createdAt),
        isMe: const Value(false),
        isAgent: const Value(false),
      ),
    );

    await _conversationsDao.updateLastSeq(msg.conversationId, msg.seq);
  }
}
