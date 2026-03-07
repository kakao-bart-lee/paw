import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/errors/app_error.dart';
import 'package:paw_client/core/http/api_client.dart';

void main() {
  group('AppErrorMapper', () {
    test('maps unauthorized ApiException to force-login ui error', () {
      final error = AppErrorMapper.map(
        ApiException.fromStatusCode(401, 'expired token'),
      );

      expect(error.code, UiErrorCode.unauthorized);
      expect(error.shouldForceLogin, isTrue);
      expect(error.message, contains('다시 로그인'));
    });

    test('maps forbidden/server/network/timeout', () {
      final forbidden = AppErrorMapper.map(
        ApiException.fromStatusCode(403, 'forbidden'),
      );
      final server = AppErrorMapper.map(
        ApiException.fromStatusCode(503, 'temporarily unavailable'),
      );
      final network = AppErrorMapper.map(ApiException.network('offline'));
      final timeout = AppErrorMapper.map(ApiException.timeout());

      expect(forbidden.code, UiErrorCode.forbidden);
      expect(server.code, UiErrorCode.server);
      expect(network.code, UiErrorCode.network);
      expect(timeout.code, UiErrorCode.timeout);
    });

    test('falls back to unknown for non-api errors', () {
      final error = AppErrorMapper.map(Exception('unexpected'));

      expect(error.code, UiErrorCode.unknown);
      expect(error.message, contains('Exception: unexpected'));
    });
  });
}
