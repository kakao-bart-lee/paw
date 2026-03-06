import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/src/rust/api.dart';
import 'package:paw_client/src/rust/frb_generated.dart';

const _runNativeFfiTests = bool.fromEnvironment(
  'RUN_NATIVE_FFI_TESTS',
  defaultValue: false,
);

void main() {
  test(
    'encrypt/decrypt round-trip via Rust FFI',
    () async {
      await RustLib.init();
      final account = createAccount();
      final plaintext = 'hello paw ffi'.codeUnits;

      final ciphertext = encrypt(
        theirSignedPrekey: account.signedPrekey,
        plaintext: plaintext,
      );
      final decrypted = decrypt(
        mySignedPrekeySecret: account.signedPrekeySecret,
        ciphertext: ciphertext,
      );

      expect(decrypted, orderedEquals(plaintext));
    },
    // Requires a built native paw_ffi library in FRB loader path.
    skip: !_runNativeFfiTests,
  );
}
