import 'package:drift/drift.dart';
import '../app_database.dart';
import '../tables/messages_table.dart';

part 'messages_dao.g.dart';

@DriftAccessor(tables: [MessagesTable])
class MessagesDao extends DatabaseAccessor<AppDatabase>
    with _$MessagesDaoMixin {
  MessagesDao(super.db);

  /// Get messages for conversation, ordered by seq
  Future<List<MessagesTableData>> getMessages(
    String conversationId, {
    int afterSeq = 0,
  }) {
    return (select(messagesTable)
          ..where((t) =>
              t.conversationId.equals(conversationId) &
              t.seq.isBiggerThanValue(afterSeq))
          ..orderBy([(t) => OrderingTerm.asc(t.seq)]))
        .get();
  }

  /// Upsert message (insert or update on conflict)
  Future<void> upsertMessage(MessagesTableCompanion message) {
    return into(messagesTable).insertOnConflictUpdate(message);
  }

  /// Get last seq for conversation (for gap-fill on reconnection)
  Future<int> getLastSeq(String conversationId) async {
    final query = selectOnly(messagesTable)
      ..addColumns([messagesTable.seq.max()])
      ..where(messagesTable.conversationId.equals(conversationId));
    final result = await query.getSingleOrNull();
    return result?.read(messagesTable.seq.max()) ?? 0;
  }
}
