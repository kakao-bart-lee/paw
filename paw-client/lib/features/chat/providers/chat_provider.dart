import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/legacy.dart' as legacy;
import 'package:uuid/uuid.dart';

import '../../../core/crypto/e2ee_service.dart';
import '../../../core/crypto/key_storage_service.dart';
import '../../../core/di/service_locator.dart';
import '../../../core/errors/app_error.dart';
import '../../../core/http/api_client.dart';
import '../../../core/proto/messages.dart';
import '../../../core/ws/ws_service.dart';
import '../models/conversation.dart';
import '../models/message.dart';

enum ResourceLoadState { loading, ready, error }

class SendMessageResult {
  final bool ok;
  final String? message;

  const SendMessageResult._({required this.ok, this.message});

  const SendMessageResult.success() : this._(ok: true);

  const SendMessageResult.failure(String message)
    : this._(ok: false, message: message);
}

final conversationsLoadStateProvider = legacy.StateProvider<ResourceLoadState>(
  (ref) => ResourceLoadState.loading,
);

final conversationsErrorProvider = legacy.StateProvider<String?>((ref) => null);

final messagesLoadStateProvider =
    legacy.StateProvider.family<ResourceLoadState, String>(
      (ref, _) => ResourceLoadState.loading,
    );

final messagesErrorProvider = legacy.StateProvider.family<String?, String>(
  (ref, _) => null,
);

class ToolRecord {
  final String tool;
  final String label;
  ToolRecord({required this.tool, required this.label});
}

class StreamingMessage {
  final String streamId;
  final String conversationId;
  final String agentId;
  final ValueNotifier<String> contentNotifier;
  bool isComplete;
  String? currentTool;
  String? currentToolLabel;
  bool toolComplete;
  final List<ToolRecord> toolHistory = [];

  StreamingMessage({
    required this.streamId,
    required this.conversationId,
    required this.agentId,
    String initialContent = '',
    this.isComplete = false,
    this.currentTool,
    this.currentToolLabel,
    this.toolComplete = false,
  }) : contentNotifier = ValueNotifier(initialContent);

  String get content => contentNotifier.value;
  set content(String value) => contentNotifier.value = value;

  void dispose() {
    contentNotifier.dispose();
  }
}

final conversationsNotifierProvider =
    NotifierProvider<ConversationsNotifier, List<Conversation>>(
      ConversationsNotifier.new,
    );

final messagesNotifierProvider =
    NotifierProvider.family<MessagesNotifier, List<Message>, String>(
      MessagesNotifier.new,
    );

class ConversationsNotifier extends Notifier<List<Conversation>> {
  ApiClient? get _apiClient =>
      getIt.isRegistered<ApiClient>() ? getIt<ApiClient>() : null;

  @override
  List<Conversation> build() {
    Future.microtask(() => unawaited(_loadConversations()));
    return const [];
  }

  Future<void> refresh() async {
    await _loadConversations();
  }

  Future<void> _loadConversations() async {
    final apiClient = _apiClient;
    ref.read(conversationsLoadStateProvider.notifier).state =
        ResourceLoadState.loading;
    ref.read(conversationsErrorProvider.notifier).state = null;

    if (apiClient == null || apiClient.accessToken == null) {
      state = const [];
      ref.read(conversationsLoadStateProvider.notifier).state =
          ResourceLoadState.ready;
      return;
    }

    try {
      final rows = await apiClient.getConversations();
      state = rows.map(_conversationFromJson).toList();
      ref.read(conversationsLoadStateProvider.notifier).state =
          ResourceLoadState.ready;
    } catch (error) {
      final uiError = AppErrorMapper.map(error);
      ref.read(conversationsErrorProvider.notifier).state = uiError.message;
      ref.read(conversationsLoadStateProvider.notifier).state =
          ResourceLoadState.error;
      state = const [];
    }
  }

  void upsertFromMessage(Message msg) {
    final idx = state.indexWhere((c) => c.id == msg.conversationId);

    if (idx == -1) {
      state = [
        Conversation(
          id: msg.conversationId,
          name: msg.conversationId,
          unreadCount: msg.isMe ? 0 : 1,
          updatedAt: msg.createdAt,
          lastMessage: msg,
        ),
        ...state,
      ];
      return;
    }

    final conv = state[idx];
    final updated = Conversation(
      id: conv.id,
      name: conv.name,
      avatarUrl: conv.avatarUrl,
      unreadCount: msg.isMe ? conv.unreadCount : conv.unreadCount + 1,
      updatedAt: msg.createdAt,
      lastMessage: msg,
      isE2ee: conv.isE2ee,
      agents: conv.agents,
    );

    final next = [...state];
    next.removeAt(idx);
    state = [updated, ...next];
  }

  Conversation _conversationFromJson(Map<String, dynamic> json) {
    final lastMessage = json['last_message'];
    return Conversation(
      id: (json['id'] ?? '').toString(),
      name: (json['name'] ?? 'Conversation').toString(),
      avatarUrl: json['avatar_url'] as String?,
      unreadCount: (json['unread_count'] as num?)?.toInt() ?? 0,
      updatedAt:
          DateTime.tryParse((json['updated_at'] ?? '').toString()) ??
          DateTime.now(),
      lastMessage: lastMessage is Map<String, dynamic>
          ? _messageFromJson(lastMessage)
          : null,
      isE2ee: json['is_e2ee'] == true,
      agents:
          (json['agents'] as List?)?.map((e) => e.toString()).toList() ??
          const [],
    );
  }
}

class MessagesNotifier extends Notifier<List<Message>> {
  MessagesNotifier(this._conversationId);

  final String _conversationId;

  ApiClient? get _apiClient =>
      getIt.isRegistered<ApiClient>() ? getIt<ApiClient>() : null;
  WsService? get _wsService =>
      getIt.isRegistered<WsService>() ? getIt<WsService>() : null;

  StreamSubscription<ServerMessage>? _wsSubscription;
  final Map<String, StreamingMessage> _activeStreams = {};

  Map<String, StreamingMessage> get activeStreams => _activeStreams;

  @override
  List<Message> build() {
    _bindWs(_conversationId);
    Future.microtask(() => unawaited(_loadMessages(_conversationId)));
    return const [];
  }

  void _bindWs(String conversationId) {
    final wsService = _wsService;
    if (wsService == null || _wsSubscription != null) {
      return;
    }

    _wsSubscription = wsService.messages.listen((msg) {
      if (msg is MessageReceivedMsg && msg.conversationId == conversationId) {
        addMessageFromWs(msg);
      } else if (msg is StreamStartMsg) {
        handleStreamStart(msg);
      } else if (msg is ContentDeltaMsg) {
        handleContentDelta(msg);
      } else if (msg is ToolStartMsg) {
        handleToolStart(msg);
      } else if (msg is ToolEndMsg) {
        handleToolEnd(msg);
      } else if (msg is StreamEndMsg) {
        handleStreamEnd(msg);
      }
    });

    ref.onDispose(() {
      _wsSubscription?.cancel();
      _wsSubscription = null;
    });

    if (wsService.isConnected) {
      final lastSeq = state.isEmpty ? 0 : state.last.seq;
      wsService.requestSync(conversationId, lastSeq);
    }
  }

  Future<void> refresh() async {
    await _loadMessages(_conversationId);
  }

  Future<void> _loadMessages(String conversationId) async {
    final apiClient = _apiClient;
    ref.read(messagesLoadStateProvider(conversationId).notifier).state =
        ResourceLoadState.loading;
    ref.read(messagesErrorProvider(conversationId).notifier).state = null;

    if (apiClient == null || apiClient.accessToken == null) {
      state = const [];
      ref.read(messagesLoadStateProvider(conversationId).notifier).state =
          ResourceLoadState.ready;
      return;
    }

    try {
      final payload = await apiClient.getMessages(conversationId);
      final rows = (payload['messages'] as List?) ?? const [];
      state =
          rows
              .whereType<Map>()
              .map((row) => _messageFromJson(Map<String, dynamic>.from(row)))
              .toList()
            ..sort((a, b) => a.seq.compareTo(b.seq));

      ref.read(messagesLoadStateProvider(conversationId).notifier).state =
          ResourceLoadState.ready;
    } catch (error) {
      final uiError = AppErrorMapper.map(error);
      ref.read(messagesErrorProvider(conversationId).notifier).state =
          uiError.message;
      ref.read(messagesLoadStateProvider(conversationId).notifier).state =
          ResourceLoadState.error;
      state = const [];
    }
  }

  void addMessage(Message msg) {
    if (state.any((m) => m.id == msg.id || m.seq == msg.seq)) {
      return;
    }

    state = [...state, msg];
    ref.read(conversationsNotifierProvider.notifier).upsertFromMessage(msg);
  }

  void addMessageFromWs(MessageReceivedMsg msg) {
    final convs = ref.read(conversationsNotifierProvider);
    Conversation? conversation;
    for (final conv in convs) {
      if (conv.id == msg.conversationId) {
        conversation = conv;
        break;
      }
    }
    final hasAgents = conversation?.agents.isNotEmpty ?? false;
    final isAgent = hasAgents && msg.senderId != 'me';

    final localMessage = Message(
      id: msg.id,
      conversationId: msg.conversationId,
      senderId: msg.senderId,
      content: msg.content,
      format: _toMessageFormat(msg.format),
      seq: msg.seq,
      createdAt: msg.createdAt,
      isMe: false,
      isAgent: isAgent,
    );

    addMessage(localMessage);
    _wsService?.sendAck(msg.conversationId, msg.seq);
  }

  void handleStreamStart(StreamStartMsg msg) {
    if (msg.conversationId != _conversationId) return;
    _activeStreams[msg.streamId] = StreamingMessage(
      streamId: msg.streamId,
      conversationId: msg.conversationId,
      agentId: msg.agentId,
    );
    ref.notifyListeners();
  }

  void handleContentDelta(ContentDeltaMsg msg) {
    final stream = _activeStreams[msg.streamId];
    if (stream != null) {
      stream.content += msg.delta;
    }
  }

  void handleToolStart(ToolStartMsg msg) {
    final stream = _activeStreams[msg.streamId];
    if (stream != null) {
      stream.currentTool = msg.tool;
      stream.currentToolLabel = msg.label;
      stream.toolComplete = false;
      stream.toolHistory.add(ToolRecord(tool: msg.tool, label: msg.label));
      ref.notifyListeners();
    }
  }

  void handleToolEnd(ToolEndMsg msg) {
    final stream = _activeStreams[msg.streamId];
    if (stream != null && stream.currentTool == msg.tool) {
      stream.toolComplete = true;
      ref.notifyListeners();
    }
  }

  void handleStreamEnd(StreamEndMsg msg) {
    final stream = _activeStreams[msg.streamId];
    if (stream != null) {
      stream.isComplete = true;

      final toolCalls = stream.toolHistory
          .map((t) => ToolCallRecord(tool: t.tool, label: t.label))
          .toList();

      final finalMessage = Message(
        id: msg.streamId,
        conversationId: stream.conversationId,
        senderId: stream.agentId,
        content: stream.content,
        format: MessageFormat.markdown,
        seq: state.isEmpty ? 1 : state.last.seq + 1,
        createdAt: DateTime.now(),
        isMe: false,
        isAgent: true,
        toolCalls: toolCalls,
      );

      _activeStreams.remove(msg.streamId);
      stream.dispose();
      addMessage(finalMessage);
    }
  }

  Future<SendMessageResult> sendMessage(String content) async {
    final apiClient = _apiClient;
    final wsService = _wsService;

    if (apiClient == null || apiClient.accessToken == null) {
      return const SendMessageResult.failure('로그인 후 메시지를 전송할 수 있습니다.');
    }

    if (wsService == null || !wsService.isConnected) {
      return const SendMessageResult.failure(
        '실시간 연결이 끊겨 있어 전송할 수 없습니다. 잠시 후 다시 시도해주세요.',
      );
    }

    String contentToSend = content;
    Conversation? conversation;
    for (final conv in ref.read(conversationsNotifierProvider)) {
      if (conv.id == _conversationId) {
        conversation = conv;
        break;
      }
    }

    if (conversation?.isE2ee == true) {
      final e2eeService = getIt.isRegistered<E2eeService>()
          ? getIt<E2eeService>()
          : null;
      final keyStorage = getIt.isRegistered<KeyStorageService>()
          ? getIt<KeyStorageService>()
          : null;

      if (e2eeService != null && keyStorage != null) {
        final keys = await keyStorage.loadKeys();
        if (keys == null) {
          developer.log(
            'E2EE enabled for $_conversationId, but no local keys; sending plaintext until key exchange is wired.',
          );
        } else {
          developer.log(
            'E2EE enabled for $_conversationId with local keys present; recipient key lookup deferred, sending plaintext placeholder.',
          );
        }
      }
    }

    final optimistic = Message(
      id: const Uuid().v4(),
      conversationId: _conversationId,
      senderId: 'me',
      content: content,
      format: MessageFormat.plain,
      seq: state.isEmpty ? 1 : state.last.seq + 1,
      createdAt: DateTime.now(),
      isMe: true,
      isAgent: false,
    );

    addMessage(optimistic);

    try {
      final serverMessage = await apiClient.sendMessage(
        _conversationId,
        contentToSend,
        const Uuid().v4(),
      );

      final confirmed = _messageFromJson(
        serverMessage,
        fallback: optimistic,
      ).copyWithMe(isMe: true, isAgent: false);
      state = [
        for (final msg in state)
          if (msg.id == optimistic.id) confirmed else msg,
      ];
      ref
          .read(conversationsNotifierProvider.notifier)
          .upsertFromMessage(confirmed);
      return const SendMessageResult.success();
    } catch (error) {
      final uiError = AppErrorMapper.map(error);
      state = state.where((msg) => msg.id != optimistic.id).toList();
      return SendMessageResult.failure(uiError.message);
    }
  }

  Message _messageFromJson(Map<String, dynamic> json, {Message? fallback}) {
    return Message(
      id: (json['id'] ?? fallback?.id ?? const Uuid().v4()).toString(),
      conversationId:
          (json['conversation_id'] ??
                  fallback?.conversationId ??
                  _conversationId)
              .toString(),
      senderId: (json['sender_id'] ?? fallback?.senderId ?? 'unknown')
          .toString(),
      content: (json['content'] ?? fallback?.content ?? '').toString(),
      format: _toMessageFormat((json['format'] ?? 'plain').toString()),
      seq: (json['seq'] as num?)?.toInt() ?? fallback?.seq ?? 0,
      createdAt:
          DateTime.tryParse((json['created_at'] ?? '').toString()) ??
          fallback?.createdAt ??
          DateTime.now(),
      isMe: fallback?.isMe ?? false,
      isAgent: fallback?.isAgent ?? false,
    );
  }
}

Message _messageFromJson(Map<String, dynamic> json) {
  return Message(
    id: (json['id'] ?? '').toString(),
    conversationId: (json['conversation_id'] ?? '').toString(),
    senderId: (json['sender_id'] ?? '').toString(),
    content: (json['content'] ?? '').toString(),
    format: _toMessageFormat((json['format'] ?? 'plain').toString()),
    seq: (json['seq'] as num?)?.toInt() ?? 0,
    createdAt:
        DateTime.tryParse((json['created_at'] ?? '').toString()) ??
        DateTime.now(),
    isMe: false,
    isAgent: false,
  );
}

MessageFormat _toMessageFormat(String format) {
  return format.toLowerCase() == 'markdown'
      ? MessageFormat.markdown
      : MessageFormat.plain;
}

extension on Message {
  Message copyWithMe({bool? isMe, bool? isAgent}) {
    return Message(
      id: id,
      conversationId: conversationId,
      senderId: senderId,
      content: content,
      format: format,
      seq: seq,
      createdAt: createdAt,
      isMe: isMe ?? this.isMe,
      isAgent: isAgent ?? this.isAgent,
      toolCalls: toolCalls,
      mediaId: mediaId,
      mediaUrl: mediaUrl,
      mediaType: mediaType,
      mediaFileName: mediaFileName,
      mediaSizeBytes: mediaSizeBytes,
    );
  }
}
