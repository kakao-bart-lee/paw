import 'package:drift/drift.dart';
import '../app_database.dart';
import '../tables/conversations_table.dart';

part 'conversations_dao.g.dart';

@DriftAccessor(tables: [ConversationsTable])
class ConversationsDao extends DatabaseAccessor<AppDatabase>
    with _$ConversationsDaoMixin {
  ConversationsDao(super.db);

  /// Get all conversations ordered by most recently updated
  Future<List<ConversationsTableData>> getAllConversations() {
    return (select(conversationsTable)
          ..orderBy([(t) => OrderingTerm.desc(t.updatedAt)]))
        .get();
  }

  /// Upsert conversation (insert or update on conflict)
  Future<void> upsertConversation(ConversationsTableCompanion conv) {
    return into(conversationsTable).insertOnConflictUpdate(conv);
  }

  /// Update the last known seq for a conversation (used in gap-fill)
  Future<void> updateLastSeq(String conversationId, int seq) {
    return (update(conversationsTable)
          ..where((t) => t.id.equals(conversationId)))
        .write(ConversationsTableCompanion(lastSeq: Value(seq)));
  }
}
