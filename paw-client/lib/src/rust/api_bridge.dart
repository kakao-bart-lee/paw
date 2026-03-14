import 'dart:typed_data';

import 'api_bridge_pure_dart.dart' as impl;

typedef AccountKeys = impl.AccountKeys;

Future<AccountKeys> createAccount() => impl.createAccount();

Future<Uint8List> encrypt({
  required List<int> theirSignedPrekey,
  required List<int> plaintext,
}) =>
    impl.encrypt(theirSignedPrekey: theirSignedPrekey, plaintext: plaintext);

Future<Uint8List> decrypt({
  required List<int> mySignedPrekeySecret,
  required List<int> ciphertext,
}) =>
    impl.decrypt(
      mySignedPrekeySecret: mySignedPrekeySecret,
      ciphertext: ciphertext,
    );
