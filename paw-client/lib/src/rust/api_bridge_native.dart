import 'dart:typed_data';

import 'api.dart' as api;

typedef AccountKeys = api.AccountKeys;

AccountKeys createAccount() => api.createAccount();

Uint8List encrypt({
  required List<int> theirSignedPrekey,
  required List<int> plaintext,
}) =>
    api.encrypt(theirSignedPrekey: theirSignedPrekey, plaintext: plaintext);

Uint8List decrypt({
  required List<int> mySignedPrekeySecret,
  required List<int> ciphertext,
}) =>
    api.decrypt(
      mySignedPrekeySecret: mySignedPrekeySecret,
      ciphertext: ciphertext,
    );
