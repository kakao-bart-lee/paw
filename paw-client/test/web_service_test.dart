import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/platform/web_service.dart';

void main() {
  group('WebService', () {
    late WebService service;

    setUp(() {
      service = WebService();
    });

    test('isWeb returns false when not running on web', () {
      // In standard Flutter test (Dart VM), kIsWeb is false.
      expect(service.isWeb, kIsWeb);
      expect(service.isWeb, isFalse);
    });

    test('getWebSocketUrl converts http to ws', () {
      final result = service.getWebSocketUrl('http://localhost:38173');
      expect(result, startsWith('ws://'));
      expect(result, contains('localhost:38173'));
    });

    test('getWebSocketUrl converts https to wss', () {
      final result = service.getWebSocketUrl('https://api.paw.chat');
      expect(result, startsWith('wss://'));
      expect(result, contains('api.paw.chat'));
    });

    test('getWebSocketUrl preserves existing ws scheme', () {
      final result = service.getWebSocketUrl('ws://localhost:38173/ws');
      expect(result, startsWith('ws://'));
      expect(result, contains('localhost:38173'));
    });

    test('getWebSocketUrl preserves existing wss scheme', () {
      final result = service.getWebSocketUrl('wss://api.paw.chat/ws');
      expect(result, startsWith('wss://'));
      expect(result, contains('api.paw.chat'));
    });

    test('getWebSocketUrl preserves path and query parameters', () {
      final result =
          service.getWebSocketUrl('http://localhost:38173/api?key=value');
      expect(result, 'ws://localhost:38173/api?key=value');
    });

    test('supportsServiceWorker returns false when not on web', () {
      // Stub returns kIsWeb, which is false in Dart VM tests.
      expect(service.supportsServiceWorker(), kIsWeb);
      expect(service.supportsServiceWorker(), isFalse);
    });
  });
}
