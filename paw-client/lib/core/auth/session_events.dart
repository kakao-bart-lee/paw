import 'dart:async';

enum SessionExpiryReason { unauthorized }

class SessionEvent {
  final SessionExpiryReason reason;

  const SessionEvent({required this.reason});
}

class SessionEvents {
  final _controller = StreamController<SessionEvent>.broadcast();

  Stream<SessionEvent> get stream => _controller.stream;

  void emitUnauthorized() {
    _controller.add(
      const SessionEvent(reason: SessionExpiryReason.unauthorized),
    );
  }

  Future<void> dispose() async {
    await _controller.close();
  }
}
