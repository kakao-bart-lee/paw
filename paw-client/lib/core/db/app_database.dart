import 'dart:io';

import 'package:drift/drift.dart';
import 'package:drift/native.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;

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
  AppDatabase() : super(_openConnection());

  @override
  int get schemaVersion => 1;

  @override
  MigrationStrategy get migration => MigrationStrategy(
        onCreate: (m) => m.createAll(),
      );
}

LazyDatabase _openConnection() {
  return LazyDatabase(() async {
    final dbFolder = await getApplicationDocumentsDirectory();
    final file = File(p.join(dbFolder.path, 'paw.db'));

    // SQLCipher encryption key from secure storage
    // Phase 1: use fixed key, Phase 2: derive from Ed25519 device key
    const encryptionKey = 'paw-phase1-dev-key';

    return NativeDatabase.createInBackground(
      file,
      setup: (db) {
        db.execute("PRAGMA key = '$encryptionKey'");
        db.execute('PRAGMA cipher_page_size = 4096');
        db.execute('PRAGMA kdf_iter = 64000');
      },
    );
  });
}
