import 'package:drift/drift.dart';

class MessagesTable extends Table {
  TextColumn get id => text()();
  TextColumn get conversationId => text()();
  TextColumn get senderId => text()();
  TextColumn get content => text()();
  TextColumn get format => text().withDefault(const Constant('markdown'))();
  IntColumn get seq => integer()();
  DateTimeColumn get createdAt => dateTime()();
  BoolColumn get isMe => boolean().withDefault(const Constant(false))();
  BoolColumn get isAgent => boolean().withDefault(const Constant(false))();

  @override
  Set<Column> get primaryKey => {id};

  @override
  List<Index> get indexes => [
    Index('messages_conv_seq', 'conversation_id, seq'),
  ];
}
