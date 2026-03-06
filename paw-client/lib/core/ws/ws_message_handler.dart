import 'dart:async';

import '../proto/messages.dart';
import 'ws_service.dart';

typedef MessageReceivedCallback = void Function(MessageReceivedMsg message);
typedef HelloErrorCallback = FutureOr<void> Function(HelloErrorMsg message);

class WsMessageHandler {
  final WsService wsService;
  final MessageReceivedCallback? onMessageReceived;
  final void Function(ServerTypingMsg msg)? onTyping;
  final void Function(PresenceUpdateMsg msg)? onPresence;
  final void Function(HelloOkMsg msg)? onHelloOk;
  final HelloErrorCallback? onHelloError;

  StreamSubscription<ServerMessage>? _subscription;

  WsMessageHandler({
    required this.wsService,
    this.onMessageReceived,
    this.onTyping,
    this.onPresence,
    this.onHelloOk,
    this.onHelloError,
  });

  void start() {
    _subscription ??= wsService.messages.listen(_handleMessage);
  }

  Future<void> stop() async {
    await _subscription?.cancel();
    _subscription = null;
  }

  void _handleMessage(ServerMessage msg) {
    switch (msg) {
      case MessageReceivedMsg message:
        onMessageReceived?.call(message);
      case ServerTypingMsg typing:
        onTyping?.call(typing);
      case PresenceUpdateMsg presence:
        onPresence?.call(presence);
      case HelloOkMsg helloOk:
        onHelloOk?.call(helloOk);
      case HelloErrorMsg helloError:
        onHelloError?.call(helloError);
      case StreamStartMsg _:
      case ContentDeltaMsg _:
      case ToolStartMsg _:
      case ToolEndMsg _:
      case StreamEndMsg _:
      case UnknownMsg _:
        // Reserved/unhandled for now.
        break;
    }
  }
}
