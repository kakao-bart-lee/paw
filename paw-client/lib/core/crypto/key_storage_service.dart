import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class E2eeKeys {
  final Uint8List identityKey;
  final Uint8List privKey;
  final Uint8List pubKey;

  const E2eeKeys({
    required this.identityKey,
    required this.privKey,
    required this.pubKey,
  });
}

class KeyStorageService {
  static const _identityKeyStorageKey = 'e2ee_identity_key';
  static const _privKeyStorageKey = 'e2ee_x25519_priv_key';
  static const _pubKeyStorageKey = 'e2ee_x25519_pub_key';

  final FlutterSecureStorage _secureStorage;

  const KeyStorageService({
    FlutterSecureStorage secureStorage = const FlutterSecureStorage(),
  }) : _secureStorage = secureStorage;

  Future<void> saveKeys({
    required Uint8List identityKey,
    required Uint8List privKey,
    required Uint8List pubKey,
  }) async {
    await _secureStorage.write(
      key: _identityKeyStorageKey,
      value: base64Encode(identityKey),
    );
    await _secureStorage.write(
      key: _privKeyStorageKey,
      value: base64Encode(privKey),
    );
    await _secureStorage.write(
      key: _pubKeyStorageKey,
      value: base64Encode(pubKey),
    );
  }

  Future<E2eeKeys?> loadKeys() async {
    final identityKeyValue = await _secureStorage.read(key: _identityKeyStorageKey);
    final privKeyValue = await _secureStorage.read(key: _privKeyStorageKey);
    final pubKeyValue = await _secureStorage.read(key: _pubKeyStorageKey);

    if (identityKeyValue == null || privKeyValue == null || pubKeyValue == null) {
      return null;
    }

    try {
      return E2eeKeys(
        identityKey: Uint8List.fromList(base64Decode(identityKeyValue)),
        privKey: Uint8List.fromList(base64Decode(privKeyValue)),
        pubKey: Uint8List.fromList(base64Decode(pubKeyValue)),
      );
    } catch (_) {
      return null;
    }
  }

  Future<void> clearKeys() async {
    await _secureStorage.delete(key: _identityKeyStorageKey);
    await _secureStorage.delete(key: _privKeyStorageKey);
    await _secureStorage.delete(key: _pubKeyStorageKey);
  }
}
