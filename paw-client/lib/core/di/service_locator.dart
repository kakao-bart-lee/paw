import 'package:get_it/get_it.dart';

import '../crypto/e2ee_service.dart';
import '../crypto/key_storage_service.dart';
import '../db/app_database.dart';
import '../db/daos/conversations_dao.dart';
import '../db/daos/messages_dao.dart';
import '../http/api_client.dart';
import '../sync/sync_service.dart';
import '../ws/reconnection_manager.dart';
import '../ws/ws_service.dart';

final getIt = GetIt.instance;

Future<void> setupServiceLocator() async {
  // Config
  const serverUrl =
      String.fromEnvironment('SERVER_URL', defaultValue: 'http://localhost:3000');

  if (!getIt.isRegistered<ApiClient>()) {
    getIt.registerSingleton<ApiClient>(ApiClient(baseUrl: serverUrl));
  }

  if (!getIt.isRegistered<AppDatabase>()) {
    getIt.registerSingleton<AppDatabase>(AppDatabase());
  }

  if (!getIt.isRegistered<MessagesDao>()) {
    getIt.registerSingleton<MessagesDao>(MessagesDao(getIt<AppDatabase>()));
  }

  if (!getIt.isRegistered<ConversationsDao>()) {
    getIt.registerSingleton<ConversationsDao>(ConversationsDao(getIt<AppDatabase>()));
  }

  if (!getIt.isRegistered<KeyStorageService>()) {
    getIt.registerSingleton<KeyStorageService>(const KeyStorageService());
  }

  if (!getIt.isRegistered<E2eeService>()) {
    getIt.registerSingleton<E2eeService>(E2eeService());
  }

  if (!getIt.isRegistered<ReconnectionManager>()) {
    getIt.registerSingleton<ReconnectionManager>(ReconnectionManager());
  }

  if (!getIt.isRegistered<WsService>()) {
    getIt.registerSingleton<WsService>(
      WsService(
        serverUrl: serverUrl,
        reconnectionManager: getIt<ReconnectionManager>(),
      ),
    );
  }

  if (!getIt.isRegistered<SyncService>()) {
    getIt.registerSingleton<SyncService>(
      SyncService(
        messagesDao: getIt<MessagesDao>(),
        conversationsDao: getIt<ConversationsDao>(),
        requestSync: getIt<WsService>().requestSync,
      ),
    );
  }

  getIt<WsService>().setSyncService(getIt<SyncService>());
}
