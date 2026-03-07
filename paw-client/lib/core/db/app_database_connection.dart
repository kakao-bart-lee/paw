import 'package:drift/drift.dart';

import 'app_database_connection_native.dart'
    if (dart.library.html) 'app_database_connection_web.dart';

QueryExecutor openConnection() => openConnectionImpl();
