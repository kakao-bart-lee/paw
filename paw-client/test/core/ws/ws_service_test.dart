import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/proto/messages.dart';
import 'package:paw_client/core/ws/reconnection_manager.dart';
import 'package:paw_client/core/ws/ws_service.dart';
import 'package:stream_channel/stream_channel.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

void main() {
  group('WsService', () {
    test(
      'stays connecting until hello_ok and gates outgoing messages',
      () async {
        final channel = _FakeWebSocketChannel();
        final service = WsService(
          serverUrl: 'http://localhost:38173',
          reconnectionManager: ReconnectionManager(),
          channelFactory: (_) => channel,
        );
        addTearDown(service.dispose);

        await service.connect('http://localhost:38173', 'token');

        expect(_stateOf(service), WsConnectionState.connecting);
        expect(service.isConnected, isFalse);
        expect(channel.sink.sentMessages, hasLength(1));
        expect(
          jsonDecode(channel.sink.sentMessages.single)['type'],
          equals('connect'),
        );

        service.send(
          const TypingMsg(
            type: 'typing_start',
            conversationId: 'conv_1',
          ).toJson(),
        );
        expect(channel.sink.sentMessages, hasLength(1));

        channel.emitJson({
          'v': 1,
          'type': 'hello_ok',
          'user_id': 'user_1',
          'server_time': DateTime.now().toIso8601String(),
        });
        await _flushEvents();

        expect(_stateOf(service), WsConnectionState.connected);
        expect(service.isConnected, isTrue);

        service.send(
          const TypingMsg(
            type: 'typing_start',
            conversationId: 'conv_1',
          ).toJson(),
        );
        expect(channel.sink.sentMessages, hasLength(2));
        expect(
          jsonDecode(channel.sink.sentMessages.last)['type'],
          equals('typing_start'),
        );
      },
    );

    test('moves to retrying and reconnects after socket close', () async {
      final firstChannel = _FakeWebSocketChannel();
      final secondChannel = _FakeWebSocketChannel();
      var connectCount = 0;

      final service = WsService(
        serverUrl: 'http://localhost:38173',
        reconnectionManager: ReconnectionManager(),
        channelFactory: (_) {
          connectCount += 1;
          return connectCount == 1 ? firstChannel : secondChannel;
        },
      );
      addTearDown(service.dispose);

      await service.connect('http://localhost:38173', 'token');
      firstChannel.emitJson({
        'v': 1,
        'type': 'hello_ok',
        'user_id': 'user_1',
        'server_time': DateTime.now().toIso8601String(),
      });
      await _flushEvents();

      await firstChannel.closeStream();
      await _flushEvents();

      expect(_stateOf(service), WsConnectionState.retrying);
      expect(service.isConnected, isFalse);

      await Future<void>.delayed(const Duration(milliseconds: 1100));
      await _flushEvents();

      expect(connectCount, 2);
      expect(_stateOf(service), WsConnectionState.connecting);
      expect(secondChannel.sink.sentMessages, hasLength(1));
      expect(
        jsonDecode(secondChannel.sink.sentMessages.single)['type'],
        equals('connect'),
      );
    });
  });
}

Future<void> _flushEvents() async {
  await Future<void>.delayed(Duration.zero);
  await Future<void>.delayed(Duration.zero);
}

WsConnectionState _stateOf(WsService service) {
  return (service.connectionState as ValueNotifier<WsConnectionState>).value;
}

class _FakeWebSocketChannel
    with StreamChannelMixin
    implements WebSocketChannel {
  final StreamController<dynamic> _controller =
      StreamController<dynamic>.broadcast();
  final _FakeWebSocketSink sink = _FakeWebSocketSink();

  void emitJson(Map<String, dynamic> payload) {
    _controller.add(jsonEncode(payload));
  }

  Future<void> closeStream() => _controller.close();

  @override
  int? get closeCode => null;

  @override
  String? get closeReason => null;

  @override
  String? get protocol => null;

  @override
  Future<void> get ready => Future.value();

  @override
  Stream<dynamic> get stream => _controller.stream;
}

class _FakeWebSocketSink implements WebSocketSink {
  final List<String> sentMessages = [];
  bool isClosed = false;

  @override
  Future<void> addStream(Stream stream) async {
    await for (final event in stream) {
      add(event);
    }
  }

  @override
  void add(event) {
    sentMessages.add(event as String);
  }

  @override
  void addError(Object error, [StackTrace? stackTrace]) {}

  @override
  Future<void> close([int? closeCode, String? closeReason]) async {
    isClosed = true;
  }

  @override
  Future<void> get done async {}
}
