import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class StoredTokens {
  final String accessToken;
  final String refreshToken;

  const StoredTokens({required this.accessToken, required this.refreshToken});
}

class TokenStorage {
  static const accessTokenKey = 'access_token';
  static const refreshTokenKey = 'refresh_token';

  final FlutterSecureStorage _storage;

  const TokenStorage({
    FlutterSecureStorage storage = const FlutterSecureStorage(),
  }) : _storage = storage;

  Future<StoredTokens?> read() async {
    final accessToken = await _storage.read(key: accessTokenKey);
    final refreshToken = await _storage.read(key: refreshTokenKey);
    if (accessToken == null || refreshToken == null) {
      return null;
    }

    return StoredTokens(accessToken: accessToken, refreshToken: refreshToken);
  }

  Future<void> write({
    required String accessToken,
    required String refreshToken,
  }) async {
    await _storage.write(key: accessTokenKey, value: accessToken);
    await _storage.write(key: refreshTokenKey, value: refreshToken);
  }

  Future<void> clear() async {
    await _storage.delete(key: accessTokenKey);
    await _storage.delete(key: refreshTokenKey);
  }
}
