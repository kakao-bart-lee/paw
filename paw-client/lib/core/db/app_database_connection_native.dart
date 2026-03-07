import 'dart:io';

import 'package:drift/drift.dart';
import 'package:drift/native.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

QueryExecutor openConnectionImpl() {
  return LazyDatabase(() async {
    final dbFolder = await getApplicationDocumentsDirectory();
    final file = File(p.join(dbFolder.path, 'paw.db'));

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
