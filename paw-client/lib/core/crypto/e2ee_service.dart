import 'dart:typed_data';

import '../../src/rust/api.dart' as rust_api;

class AccountKeys {
  final Uint8List identityKey;
  final Uint8List x25519PubKey;

  const AccountKeys({
    required this.identityKey,
    required this.x25519PubKey,
  });
}

class E2eeService {
  bool _isE2eeAvailable = true;

  bool get isE2eeAvailable => _isE2eeAvailable;

  AccountKeys? createAccount() {
    try {
      final keys = rust_api.createAccount();
      _isE2eeAvailable = true;
      return AccountKeys(
        identityKey: keys.identityKey,
        x25519PubKey: keys.signedPrekey,
      );
    } catch (_) {
      _isE2eeAvailable = false;
      return null;
    }
  }

  Uint8List? encryptForRecipient({
    required Uint8List recipientPubKey,
    required Uint8List plaintext,
  }) {
    try {
      final ciphertext = rust_api.encrypt(
        theirSignedPrekey: recipientPubKey,
        plaintext: plaintext,
      );
      _isE2eeAvailable = true;
      return ciphertext;
    } catch (_) {
      _isE2eeAvailable = false;
      return null;
    }
  }

  Uint8List? decryptFromSender({
    required Uint8List myPrivKey,
    required Uint8List ciphertext,
  }) {
    try {
      final plaintext = rust_api.decrypt(
        mySignedPrekeySecret: myPrivKey,
        ciphertext: ciphertext,
      );
      _isE2eeAvailable = true;
      return plaintext;
    } catch (_) {
      _isE2eeAvailable = false;
      return null;
    }
  }
}
