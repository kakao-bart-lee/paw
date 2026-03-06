import 'package:get_it/get_it.dart';

final getIt = GetIt.instance;

Future<void> setupServiceLocator() async {
  // Phase 1: Register services as they are implemented
  // Services will be registered here in subsequent tasks:
  // - T5: AuthService (OTP + Ed25519)
  // - T6: WebSocketService
  // - T9: MessageRepository
  // - T14: LocalDatabase (Drift + SQLCipher)
}
