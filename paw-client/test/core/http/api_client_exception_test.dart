import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/http/api_client.dart';

void main() {
  group('ApiException.fromStatusCode', () {
    test('maps 401 to unauthorized and invokes callback', () async {
      var called = false;

      final exception = ApiException.fromStatusCode(
        401,
        'unauthorized',
        onUnauthorized: () async {
          called = true;
        },
      );

      // Callback is called asynchronously via unawaited.
      await Future<void>.delayed(Duration.zero);

      expect(exception.kind, ApiErrorKind.unauthorized);
      expect(exception.statusCode, 401);
      expect(called, isTrue);
    });

    test('maps 403 to forbidden', () {
      final exception = ApiException.fromStatusCode(403, 'forbidden');
      expect(exception.kind, ApiErrorKind.forbidden);
    });

    test('maps 500 to server', () {
      final exception = ApiException.fromStatusCode(500, 'server error');
      expect(exception.kind, ApiErrorKind.server);
    });

    test('maps 400 to client', () {
      final exception = ApiException.fromStatusCode(400, 'bad request');
      expect(exception.kind, ApiErrorKind.client);
    });

    test('maps timeout and network factories', () {
      final timeout = ApiException.timeout();
      final network = ApiException.network('socket closed');

      expect(timeout.kind, ApiErrorKind.timeout);
      expect(network.kind, ApiErrorKind.network);
    });
  });
}
