import 'dart:typed_data';

import 'api_bridge_native.dart'
    if (dart.library.html) 'api_bridge_web.dart' as impl;

typedef AccountKeys = impl.AccountKeys;

AccountKeys createAccount() => impl.createAccount();

Uint8List encrypt({
  required List<int> theirSignedPrekey,
  required List<int> plaintext,
}) =>
    impl.encrypt(theirSignedPrekey: theirSignedPrekey, plaintext: plaintext);

Uint8List decrypt({
  required List<int> mySignedPrekeySecret,
  required List<int> ciphertext,
}) =>
    impl.decrypt(
      mySignedPrekeySecret: mySignedPrekeySecret,
      ciphertext: ciphertext,
    );
