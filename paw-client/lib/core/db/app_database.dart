import 'package:drift/drift.dart';
import 'app_database_connection.dart';

import 'tables/messages_table.dart';
import 'tables/conversations_table.dart';
import 'daos/messages_dao.dart';
import 'daos/conversations_dao.dart';

part 'app_database.g.dart';

@DriftDatabase(
  tables: [MessagesTable, ConversationsTable],
  daos: [MessagesDao, ConversationsDao],
)
class AppDatabase extends _$AppDatabase {
  AppDatabase() : super(openConnection());

  /// Test constructor — accepts an in-memory or custom executor.
  AppDatabase.forTesting(super.e);

  @override
  int get schemaVersion => 2;

  @override
  MigrationStrategy get migration => MigrationStrategy(
        onCreate: (m) async {
          await m.createAll();
          await _createFts5Tables(m);
        },
        onUpgrade: (m, from, to) async {
          if (from < 2) {
            await _createFts5Tables(m);
          }
        },
      );

  /// Creates FTS5 virtual table and sync triggers for full-text search.
  Future<void> _createFts5Tables(Migrator m) async {
    // FTS5 virtual table backed by messages_table content
    await customStatement('''
      CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
        content,
        content='messages_table',
        content_rowid='rowid',
        tokenize='unicode61'
      )
    ''');

    // Keep FTS index in sync: INSERT
    await customStatement('''
      CREATE TRIGGER IF NOT EXISTS messages_fts_ai
      AFTER INSERT ON messages_table BEGIN
        INSERT INTO messages_fts(rowid, content)
        VALUES (new.rowid, new.content);
      END
    ''');

    // Keep FTS index in sync: DELETE
    await customStatement('''
      CREATE TRIGGER IF NOT EXISTS messages_fts_ad
      AFTER DELETE ON messages_table BEGIN
        INSERT INTO messages_fts(messages_fts, rowid, content)
        VALUES ('delete', old.rowid, old.content);
      END
    ''');

    // Keep FTS index in sync: UPDATE
    await customStatement('''
      CREATE TRIGGER IF NOT EXISTS messages_fts_au
      AFTER UPDATE ON messages_table BEGIN
        INSERT INTO messages_fts(messages_fts, rowid, content)
        VALUES ('delete', old.rowid, old.content);
        INSERT INTO messages_fts(rowid, content)
        VALUES (new.rowid, new.content);
      END
    ''');
  }

  /// Rebuild the FTS index from scratch (useful after bulk imports).
  Future<void> rebuildFtsIndex() async {
    await customStatement(
      "INSERT INTO messages_fts(messages_fts) VALUES ('rebuild')",
    );
  }
}
