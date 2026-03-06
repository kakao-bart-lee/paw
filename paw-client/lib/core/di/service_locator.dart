import 'package:get_it/get_it.dart';

import '../http/api_client.dart';
import '../ws/ws_service.dart';

final getIt = GetIt.instance;

Future<void> setupServiceLocator() async {
  // Config
  const serverUrl =
      String.fromEnvironment('SERVER_URL', defaultValue: 'http://localhost:3000');

  if (!getIt.isRegistered<ApiClient>()) {
    getIt.registerSingleton<ApiClient>(ApiClient(baseUrl: serverUrl));
  }

  if (!getIt.isRegistered<WsService>()) {
    getIt.registerSingleton<WsService>(WsService(serverUrl: serverUrl));
  }
}
