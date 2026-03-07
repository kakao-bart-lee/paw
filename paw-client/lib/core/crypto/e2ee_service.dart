import 'dart:typed_data';

import '../../src/rust/api_bridge.dart' as rust_api;

/// Local representation of E2EE account keys returned from paw-ffi.
/// signedPrekey = X25519 public key (share with server)
/// signedPrekeySecret = X25519 private key (keep local, never upload)
class AccountKeys {
  final Uint8List identityKey;
  final Uint8List identitySecret;
  final Uint8List x25519PubKey;
  final Uint8List x25519PrivKey;

  const AccountKeys({
    required this.identityKey,
    required this.identitySecret,
    required this.x25519PubKey,
    required this.x25519PrivKey,
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
        identitySecret: keys.identitySecret,
        x25519PubKey: keys.signedPrekey,
        x25519PrivKey: keys.signedPrekeySecret,
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
