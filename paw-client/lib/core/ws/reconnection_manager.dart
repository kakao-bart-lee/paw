import 'dart:async';

typedef ReconnectCallback = FutureOr<void> Function();

class ReconnectionManager {
  static const int _maxAttempts = 10;
  static const List<Duration> _retryDelays = [
    Duration(seconds: 1),
    Duration(seconds: 2),
    Duration(seconds: 4),
    Duration(seconds: 8),
    Duration(seconds: 16),
    Duration(seconds: 30),
  ];

  Timer? _timer;
  int _attempts = 0;

  int get attempts => _attempts;
  bool get canRetry => _attempts < _maxAttempts;

  void scheduleReconnect(ReconnectCallback callback) {
    if (!canRetry) {
      return;
    }

    final delayIndex = _attempts < _retryDelays.length
        ? _attempts
        : _retryDelays.length - 1;
    final delay = _retryDelays[delayIndex];

    _attempts += 1;
    _timer?.cancel();
    _timer = Timer(delay, () {
      callback();
    });
  }

  void onConnected() {
    _attempts = 0;
    _timer?.cancel();
    _timer = null;
  }

  void dispose() {
    _timer?.cancel();
    _timer = null;
  }
}
