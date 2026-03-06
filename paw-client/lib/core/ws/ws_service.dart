import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

import '../proto/messages.dart';
import '../sync/sync_service.dart';
import 'reconnection_manager.dart';

class WsService {
  final String serverUrl;
  final ReconnectionManager _reconnectionManager;

  WebSocketChannel? _channel;
  StreamSubscription<dynamic>? _subscription;
  SyncService? _syncService;

  bool _connected = false;
  bool _manualDisconnect = false;
  String? _accessToken;

  final _messageController = StreamController<ServerMessage>.broadcast();
  Stream<ServerMessage> get messages => _messageController.stream;

  bool get isConnected => _connected;

  WsService({
    required this.serverUrl,
    required ReconnectionManager reconnectionManager,
  }) : _reconnectionManager = reconnectionManager;

  void setSyncService(SyncService syncService) {
    _syncService = syncService;
  }

  Future<void> connect(String serverUrl, String accessToken) async {
    _accessToken = accessToken;
    _manualDisconnect = false;

    await _subscription?.cancel();
    await _channel?.sink.close();
    _reconnectionManager.dispose();

    final uri = _buildWsUri(serverUrl, accessToken);
    _channel = WebSocketChannel.connect(uri);
    _connected = true;

    _subscription = _channel!.stream.listen(
      _onMessage,
      onError: _onError,
      onDone: _onDone,
      cancelOnError: false,
    );

    send(ConnectMsg(token: accessToken).toJson());
  }

  Future<void> connectWithStoredToken() async {
    final token = _accessToken;
    if (token == null || token.isEmpty) {
      return;
    }

    await connect(serverUrl, token);
  }

  void _onMessage(dynamic data) {
    try {
      final json = jsonDecode(data as String) as Map<String, dynamic>;
      final msg = parseServerMessage(json);
      _messageController.add(msg);

      if (msg is HelloOkMsg) {
        _reconnectionManager.onConnected();
        unawaited(_syncService?.syncAllConversations());
      }
    } catch (error, stackTrace) {
      _messageController.addError(error, stackTrace);
    }
  }

  void _onError(Object error) {
    _connected = false;
    _messageController.addError(error);
    _scheduleReconnect();
  }

  void _onDone() {
    _connected = false;
    _scheduleReconnect();
  }

  void _scheduleReconnect() {
    if (_manualDisconnect) {
      return;
    }

    final token = _accessToken;
    if (token == null || token.isEmpty) {
      return;
    }

    _reconnectionManager.scheduleReconnect(() async {
      await connect(serverUrl, token);
    });
  }

  void send(Map<String, dynamic> message) {
    if (!_connected) {
      return;
    }
    _channel?.sink.add(jsonEncode(message));
  }

  void sendTypingStart(String conversationId) {
    send(
      TypingMsg(type: 'typing_start', conversationId: conversationId).toJson(),
    );
  }

  void sendTypingStop(String conversationId) {
    send(
      TypingMsg(type: 'typing_stop', conversationId: conversationId).toJson(),
    );
  }

  void sendAck(String conversationId, int lastSeq) {
    send(MessageAckMsg(conversationId: conversationId, lastSeq: lastSeq).toJson());
  }

  void requestSync(String conversationId, int lastSeq) {
    send(SyncMsg(conversationId: conversationId, lastSeq: lastSeq).toJson());
  }

  Future<void> disconnect() async {
    _manualDisconnect = true;
    _reconnectionManager.dispose();
    await _subscription?.cancel();
    await _channel?.sink.close();
    _subscription = null;
    _channel = null;
    _connected = false;
  }

  Future<void> dispose() async {
    await disconnect();
    await _messageController.close();
  }

  Uri _buildWsUri(String rawServerUrl, String token) {
    final base = Uri.parse(rawServerUrl);
    final isSecure = base.scheme == 'https' || base.scheme == 'wss';
    return base.replace(
      scheme: isSecure ? 'wss' : 'ws',
      path: '/ws',
      queryParameters: {'token': token},
    );
  }
}
