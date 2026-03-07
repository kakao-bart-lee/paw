import 'dart:typed_data';

class AccountKeys {
  final Uint8List identityKey;
  final Uint8List identitySecret;
  final Uint8List signedPrekey;
  final Uint8List signedPrekeySecret;

  const AccountKeys({
    required this.identityKey,
    required this.identitySecret,
    required this.signedPrekey,
    required this.signedPrekeySecret,
  });
}

AccountKeys createAccount() {
  throw UnsupportedError('Rust FFI is not available on web');
}

Uint8List encrypt({
  required List<int> theirSignedPrekey,
  required List<int> plaintext,
}) {
  throw UnsupportedError('Rust FFI is not available on web');
}

Uint8List decrypt({
  required List<int> mySignedPrekeySecret,
  required List<int> ciphertext,
}) {
  throw UnsupportedError('Rust FFI is not available on web');
}
