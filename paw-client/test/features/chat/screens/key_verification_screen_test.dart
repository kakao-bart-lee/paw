import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/features/chat/screens/key_verification_screen.dart';

void main() {
  group('E2EE verification support', () {
    test('returns unsupported when web override is true', () {
      expect(isE2eeVerificationSupported(isWebOverride: true), isFalse);
    });

    test('returns supported when web override is false', () {
      expect(isE2eeVerificationSupported(isWebOverride: false), isTrue);
    });
  });
}
