import 'dart:async';
import 'dart:convert';

import '../crypto/e2ee_service.dart';
import '../crypto/key_storage_service.dart';
import '../di/service_locator.dart';
import '../proto/messages.dart';
import 'ws_service.dart';

typedef MessageReceivedCallback = void Function(MessageReceivedMsg message);
typedef HelloErrorCallback = FutureOr<void> Function(HelloErrorMsg message);

class WsMessageHandler {
  static final RegExp _ciphertextBase64Pattern = RegExp(r'^[A-Za-z0-9+/=]+$');

  final WsService wsService;
  final MessageReceivedCallback? onMessageReceived;
  final void Function(ServerTypingMsg msg)? onTyping;
  final void Function(PresenceUpdateMsg msg)? onPresence;
  final void Function(HelloOkMsg msg)? onHelloOk;
  final HelloErrorCallback? onHelloError;
  final void Function(StreamStartMsg msg)? onStreamStart;
  final void Function(ContentDeltaMsg msg)? onContentDelta;
  final void Function(ToolStartMsg msg)? onToolStart;
  final void Function(ToolEndMsg msg)? onToolEnd;
  final void Function(StreamEndMsg msg)? onStreamEnd;

  StreamSubscription<ServerMessage>? _subscription;

  WsMessageHandler({
    required this.wsService,
    this.onMessageReceived,
    this.onTyping,
    this.onPresence,
    this.onHelloOk,
    this.onHelloError,
    this.onStreamStart,
    this.onContentDelta,
    this.onToolStart,
    this.onToolEnd,
    this.onStreamEnd,
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
        unawaited(_handleMessageReceived(message));
      case ServerTypingMsg typing:
        onTyping?.call(typing);
      case PresenceUpdateMsg presence:
        onPresence?.call(presence);
      case HelloOkMsg helloOk:
        onHelloOk?.call(helloOk);
      case HelloErrorMsg helloError:
        onHelloError?.call(helloError);
      case StreamStartMsg streamStart:
        onStreamStart?.call(streamStart);
      case ContentDeltaMsg contentDelta:
        onContentDelta?.call(contentDelta);
      case ToolStartMsg toolStart:
        onToolStart?.call(toolStart);
      case ToolEndMsg toolEnd:
        onToolEnd?.call(toolEnd);
      case StreamEndMsg streamEnd:
        onStreamEnd?.call(streamEnd);
      case UnknownMsg _:
        // Reserved/unhandled for now.
        break;
    }
  }

  Future<void> _handleMessageReceived(MessageReceivedMsg message) async {
    var outgoingMessage = message;
    final e2eeService =
        getIt.isRegistered<E2eeService>() ? getIt<E2eeService>() : null;
    final keyStorage =
        getIt.isRegistered<KeyStorageService>() ? getIt<KeyStorageService>() : null;

    if (e2eeService != null &&
        keyStorage != null &&
        _looksLikeCiphertext(message.content)) {
      try {
        final keys = await keyStorage.loadKeys();
        if (keys != null) {
          final ciphertext = base64Decode(message.content);
          final decrypted = await e2eeService.decryptFromSender(
            myPrivKey: keys.privKey,
            ciphertext: ciphertext,
          );
          if (decrypted != null) {
            outgoingMessage = MessageReceivedMsg(
              v: message.v,
              id: message.id,
              conversationId: message.conversationId,
              senderId: message.senderId,
              content: utf8.decode(decrypted, allowMalformed: true),
              format: message.format,
              seq: message.seq,
              createdAt: message.createdAt,
              blocks: message.blocks,
            );
          }
        }
      } catch (_) {
        // Decryption is best-effort only; keep original content.
      }
    }

    onMessageReceived?.call(outgoingMessage);
  }

  bool _looksLikeCiphertext(String content) {
    return content.length > 44 && _ciphertextBase64Pattern.hasMatch(content);
  }
}
