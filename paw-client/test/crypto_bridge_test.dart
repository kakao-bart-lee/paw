import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/src/rust/api_bridge.dart';

void main() {
  test('encrypt/decrypt round-trip via pure Dart bridge', () async {
    final account = await createAccount();
    final plaintext = 'hello paw client'.codeUnits;

    final ciphertext = await encrypt(
      theirSignedPrekey: account.signedPrekey,
      plaintext: plaintext,
    );
    final decrypted = await decrypt(
      mySignedPrekeySecret: account.signedPrekeySecret,
      ciphertext: ciphertext,
    );

    expect(decrypted, orderedEquals(plaintext));
  });
}
