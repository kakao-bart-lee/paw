import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/ws/reconnection_manager.dart';

void main() {
  group('ReconnectionManager', () {
    test('uses exponential delays on early retries', () {
      fakeAsync((async) {
        final manager = ReconnectionManager();
        var called = 0;

        manager.scheduleReconnect(() {
          called += 1;
        });

        async.elapse(const Duration(milliseconds: 999));
        expect(called, 0);

        async.elapse(const Duration(milliseconds: 1));
        expect(called, 1);

        manager.scheduleReconnect(() {
          called += 1;
        });
        async.elapse(const Duration(seconds: 1));
        expect(called, 1);

        async.elapse(const Duration(seconds: 1));
        expect(called, 2);
      });
    });

    test('stops scheduling after max attempts', () {
      fakeAsync((async) {
        final manager = ReconnectionManager();
        var called = 0;

        for (var i = 0; i < 12; i++) {
          manager.scheduleReconnect(() {
            called += 1;
          });
        }

        expect(manager.attempts, 10);
        expect(manager.canRetry, isFalse);

        async.elapse(const Duration(minutes: 2));
        // only last scheduled callback should fire once.
        expect(called, 1);
      });
    });

    test('onConnected resets attempts and cancels pending timer', () {
      fakeAsync((async) {
        final manager = ReconnectionManager();
        var called = 0;

        manager.scheduleReconnect(() {
          called += 1;
        });
        expect(manager.attempts, 1);

        manager.onConnected();
        expect(manager.attempts, 0);
        expect(manager.canRetry, isTrue);

        async.elapse(const Duration(seconds: 2));
        expect(called, 0);
      });
    });
  });
}
