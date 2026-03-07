import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class StoredTokens {
  final String accessToken;
  final String refreshToken;

  const StoredTokens({required this.accessToken, required this.refreshToken});
}

class TokenStorage {
  static const accessTokenKey = 'access_token';
  static const refreshTokenKey = 'refresh_token';
  static StoredTokens? _memoryTokens;

  final FlutterSecureStorage _storage;

  const TokenStorage({
    FlutterSecureStorage storage = const FlutterSecureStorage(),
  }) : _storage = storage;

  Future<StoredTokens?> read() async {
    try {
      final accessToken = await _storage.read(key: accessTokenKey);
      final refreshToken = await _storage.read(key: refreshTokenKey);
      if (accessToken == null || refreshToken == null) {
        return _memoryTokens;
      }

      final tokens = StoredTokens(
        accessToken: accessToken,
        refreshToken: refreshToken,
      );
      _memoryTokens = tokens;
      return tokens;
    } catch (_) {
      return _memoryTokens;
    }
  }

  Future<void> write({
    required String accessToken,
    required String refreshToken,
  }) async {
    final tokens = StoredTokens(
      accessToken: accessToken,
      refreshToken: refreshToken,
    );
    _memoryTokens = tokens;
    try {
      await _storage.write(key: accessTokenKey, value: accessToken);
      await _storage.write(key: refreshTokenKey, value: refreshToken);
    } catch (_) {
      // Best-effort: allow session to continue in-memory when keychain is blocked.
    }
  }

  Future<void> clear() async {
    _memoryTokens = null;
    try {
      await _storage.delete(key: accessTokenKey);
      await _storage.delete(key: refreshTokenKey);
    } catch (_) {
      // Best-effort clear for environments where secure storage is unavailable.
    }
  }
}
