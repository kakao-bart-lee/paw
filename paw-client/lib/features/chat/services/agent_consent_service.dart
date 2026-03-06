import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class AgentConsentService {
  final FlutterSecureStorage _storage;

  AgentConsentService({FlutterSecureStorage? storage})
      : _storage = storage ?? const FlutterSecureStorage();

  String _getKey(String conversationId, String agentId) {
    return 'consent_${conversationId}_$agentId';
  }

  /// Returns true if the user has consented, false if declined, and null if no decision has been made.
  Future<bool?> getConsent(String conversationId, String agentId) async {
    final value = await _storage.read(key: _getKey(conversationId, agentId));
    if (value == null) return null;
    return value == 'true';
  }

  /// Checks if the user has consented. Returns false if declined or no decision.
  Future<bool> hasConsented(String conversationId, String agentId) async {
    final value = await getConsent(conversationId, agentId);
    return value == true;
  }

  /// Saves the consent state to secure storage.
  Future<void> setConsent(String conversationId, String agentId, bool consented) async {
    await _storage.write(
      key: _getKey(conversationId, agentId),
      value: consented.toString(),
    );
  }
}
