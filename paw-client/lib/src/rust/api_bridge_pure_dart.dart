import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

const _hkdfInfo = 'paw-ffi-aes-256-gcm-key';
const _nonceLength = 12;
const _publicKeyLength = 32;
const _macLength = 16;

final _x25519 = X25519();
final _aesGcm = AesGcm.with256bits();
final _hkdf = Hkdf(hmac: Hmac.sha256(), outputLength: 32);

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

Future<AccountKeys> createAccount() async {
  final identityKeyPair = await _x25519.newKeyPair();
  final signedPrekeyPair = await _x25519.newKeyPair();

  final identityPublicKey = await identityKeyPair.extractPublicKey();
  final signedPrekeyPublicKey = await signedPrekeyPair.extractPublicKey();

  return AccountKeys(
    identityKey: Uint8List.fromList(identityPublicKey.bytes),
    identitySecret: Uint8List.fromList(
      await identityKeyPair.extractPrivateKeyBytes(),
    ),
    signedPrekey: Uint8List.fromList(signedPrekeyPublicKey.bytes),
    signedPrekeySecret: Uint8List.fromList(
      await signedPrekeyPair.extractPrivateKeyBytes(),
    ),
  );
}

Future<Uint8List> encrypt({
  required List<int> theirSignedPrekey,
  required List<int> plaintext,
}) async {
  final remotePublicKey = SimplePublicKey(
    Uint8List.fromList(theirSignedPrekey),
    type: KeyPairType.x25519,
  );
  final ephemeralKeyPair = await _x25519.newKeyPair();
  final ephemeralPublicKey = await ephemeralKeyPair.extractPublicKey();

  final sharedSecret = await _x25519.sharedSecretKey(
    keyPair: ephemeralKeyPair,
    remotePublicKey: remotePublicKey,
  );
  final derivedKey = await _hkdf.deriveKey(
    secretKey: sharedSecret,
    info: utf8.encode(_hkdfInfo),
  );
  final secretBox = await _aesGcm.encrypt(
    plaintext,
    secretKey: derivedKey,
    nonce: _aesGcm.newNonce(),
  );

  return Uint8List.fromList([
    ...secretBox.nonce,
    ...ephemeralPublicKey.bytes,
    ...secretBox.cipherText,
    ...secretBox.mac.bytes,
  ]);
}

Future<Uint8List> decrypt({
  required List<int> mySignedPrekeySecret,
  required List<int> ciphertext,
}) async {
  final bytes = Uint8List.fromList(ciphertext);
  if (bytes.length < _nonceLength + _publicKeyLength + _macLength) {
    throw ArgumentError('ciphertext too short');
  }

  final nonce = bytes.sublist(0, _nonceLength);
  final publicKeyBytes = bytes.sublist(
    _nonceLength,
    _nonceLength + _publicKeyLength,
  );
  final cipherTextBytes = bytes.sublist(
    _nonceLength + _publicKeyLength,
    bytes.length - _macLength,
  );
  final macBytes = bytes.sublist(bytes.length - _macLength);

  final privateKeyBytes = Uint8List.fromList(mySignedPrekeySecret);
  final publicKey = await _x25519
      .newKeyPairFromSeed(privateKeyBytes)
      .then((pair) => pair.extractPublicKey());
  final localKeyPair = SimpleKeyPairData(
    privateKeyBytes,
    publicKey: publicKey,
    type: KeyPairType.x25519,
  );

  final sharedSecret = await _x25519.sharedSecretKey(
    keyPair: localKeyPair,
    remotePublicKey: SimplePublicKey(
      publicKeyBytes,
      type: KeyPairType.x25519,
    ),
  );
  final derivedKey = await _hkdf.deriveKey(
    secretKey: sharedSecret,
    info: utf8.encode(_hkdfInfo),
  );

  final clearText = await _aesGcm.decrypt(
    SecretBox(
      cipherTextBytes,
      nonce: nonce,
      mac: Mac(macBytes),
    ),
    secretKey: derivedKey,
  );

  return Uint8List.fromList(clearText);
}
