import 'package:drift/drift.dart';

class ConversationsTable extends Table {
  TextColumn get id => text()();
  TextColumn get name => text()();
  TextColumn get avatarUrl => text().nullable()();
  IntColumn get lastSeq => integer().withDefault(const Constant(0))();
  IntColumn get unreadCount => integer().withDefault(const Constant(0))();
  DateTimeColumn get updatedAt => dateTime()();

  @override
  Set<Column> get primaryKey => {id};
}
