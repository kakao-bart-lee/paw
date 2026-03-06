import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:uuid/uuid.dart';

import '../../../core/di/service_locator.dart';
import '../../../core/http/api_client.dart';
import '../../../core/proto/messages.dart';
import '../../../core/ws/ws_service.dart';
import '../models/conversation.dart';
import '../models/message.dart';

final _mockMessagesData = <String, List<Message>>{
  'conv_1': [
    Message(
      id: 'msg_1',
      conversationId: 'conv_1',
      senderId: 'other_1',
      content: '안녕하세요! Paw 메신저에 오신 것을 환영합니다.',
      format: MessageFormat.plain,
      seq: 1,
      createdAt: DateTime.now().subtract(const Duration(minutes: 10)),
      isMe: false,
      isAgent: false,
    ),
    Message(
      id: 'msg_2',
      conversationId: 'conv_1',
      senderId: 'me',
      content: '반갑습니다. AI 에이전트 기능은 어떻게 사용하나요?',
      format: MessageFormat.plain,
      seq: 2,
      createdAt: DateTime.now().subtract(const Duration(minutes: 9)),
      isMe: true,
      isAgent: false,
    ),
    Message(
      id: 'msg_3',
      conversationId: 'conv_1',
      senderId: 'agent_1',
      content: '제가 도와드릴게요! 궁금한 점을 물어보시면 답변해 드립니다.',
      format: MessageFormat.plain,
      seq: 3,
      createdAt: DateTime.now().subtract(const Duration(minutes: 8)),
      isMe: false,
      isAgent: true,
    ),
    Message(
      id: 'msg_4',
      conversationId: 'conv_1',
      senderId: 'me',
      content: '오, 신기하네요. 감사합니다.',
      format: MessageFormat.plain,
      seq: 4,
      createdAt: DateTime.now().subtract(const Duration(minutes: 7)),
      isMe: true,
      isAgent: false,
    ),
    Message(
      id: 'msg_5',
      conversationId: 'conv_1',
      senderId: 'other_1',
      content: '앞으로 자주 이용해주세요!',
      format: MessageFormat.plain,
      seq: 5,
      createdAt: DateTime.now().subtract(const Duration(minutes: 6)),
      isMe: false,
      isAgent: false,
    ),
  ],
  'conv_2': [
    Message(
      id: 'msg_6',
      conversationId: 'conv_2',
      senderId: 'other_2',
      content: '오늘 회의 시간 언제가 좋으신가요?',
      format: MessageFormat.plain,
      seq: 1,
      createdAt: DateTime.now().subtract(const Duration(hours: 1)),
      isMe: false,
      isAgent: false,
    ),
  ],
  'conv_3': [
    Message(
      id: 'msg_7',
      conversationId: 'conv_3',
      senderId: 'me',
      content: '프로젝트 일정 확인 부탁드립니다.',
      format: MessageFormat.plain,
      seq: 1,
      createdAt: DateTime.now().subtract(const Duration(days: 1)),
      isMe: true,
      isAgent: false,
    ),
  ],
};

final _mockConversations = [
  Conversation(
    id: 'conv_1',
    name: 'Paw 공식 지원팀',
    unreadCount: 0,
    updatedAt: DateTime.now().subtract(const Duration(minutes: 6)),
    lastMessage: _mockMessagesData['conv_1']!.last,
  ),
  Conversation(
    id: 'conv_2',
    name: '개발팀',
    unreadCount: 1,
    updatedAt: DateTime.now().subtract(const Duration(hours: 1)),
    lastMessage: _mockMessagesData['conv_2']!.last,
  ),
  Conversation(
    id: 'conv_3',
    name: '디자인팀',
    unreadCount: 0,
    updatedAt: DateTime.now().subtract(const Duration(days: 1)),
    lastMessage: _mockMessagesData['conv_3']!.last,
  ),
];

final conversationsNotifierProvider =
    NotifierProvider<ConversationsNotifier, List<Conversation>>(
  ConversationsNotifier.new,
);

final messagesNotifierProvider =
    NotifierProviderFamily<MessagesNotifier, List<Message>, String>(
  MessagesNotifier.new,
);

class ConversationsNotifier extends Notifier<List<Conversation>> {
  ApiClient? get _apiClient =>
      getIt.isRegistered<ApiClient>() ? getIt<ApiClient>() : null;

  @override
  List<Conversation> build() {
    unawaited(_loadConversations());
    return _mockConversations;
  }

  Future<void> _loadConversations() async {
    final apiClient = _apiClient;
    if (apiClient == null) {
      return;
    }

    try {
      final rows = await apiClient.getConversations();
      if (rows.isEmpty) {
        return;
      }

      final next = rows.map(_conversationFromJson).toList();
      state = next;
    } catch (_) {
      // Keep mock fallback when network fails.
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
      updatedAt: DateTime.tryParse((json['updated_at'] ?? '').toString()) ??
          DateTime.now(),
      lastMessage: lastMessage is Map<String, dynamic>
          ? _messageFromJson(lastMessage)
          : null,
    );
  }
}

class MessagesNotifier extends FamilyNotifier<List<Message>, String> {
  ApiClient? get _apiClient =>
      getIt.isRegistered<ApiClient>() ? getIt<ApiClient>() : null;
  WsService? get _wsService =>
      getIt.isRegistered<WsService>() ? getIt<WsService>() : null;

  StreamSubscription<ServerMessage>? _wsSubscription;
  late final String _conversationId;

  @override
  List<Message> build(String conversationId) {
    _conversationId = conversationId;
    _bindWs(conversationId);
    unawaited(_loadMessages(conversationId));
    return _mockMessagesData[conversationId] ?? [];
  }

  void _bindWs(String conversationId) {
    final wsService = _wsService;
    if (wsService == null || _wsSubscription != null) {
      return;
    }

    _wsSubscription = wsService.messages.listen((msg) {
      if (msg is MessageReceivedMsg && msg.conversationId == conversationId) {
        addMessageFromWs(msg);
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

  Future<void> _loadMessages(String conversationId) async {
    final apiClient = _apiClient;
    if (apiClient == null) {
      return;
    }

    try {
      final payload = await apiClient.getMessages(conversationId);
      final rows = (payload['messages'] as List?) ?? const [];
      if (rows.isEmpty) {
        return;
      }

      state = rows
          .whereType<Map>()
          .map((row) => _messageFromJson(Map<String, dynamic>.from(row)))
          .toList()
        ..sort((a, b) => a.seq.compareTo(b.seq));
    } catch (_) {
      // Keep mock fallback.
    }
  }

  void addMessage(Message msg) {
    if (state.any((m) => m.id == msg.id)) {
      return;
    }

    state = [...state, msg];
    ref.read(conversationsNotifierProvider.notifier).upsertFromMessage(msg);
  }

  void addMessageFromWs(MessageReceivedMsg msg) {
    final localMessage = Message(
      id: msg.id,
      conversationId: msg.conversationId,
      senderId: msg.senderId,
      content: msg.content,
      format: _toMessageFormat(msg.format),
      seq: msg.seq,
      createdAt: msg.createdAt,
      isMe: false,
      isAgent: false,
    );

    addMessage(localMessage);
    _wsService?.sendAck(msg.conversationId, msg.seq);
  }

  Future<void> sendMessage(String content) async {
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

    final apiClient = _apiClient;
    if (apiClient == null) {
      return;
    }

    try {
      final serverMessage = await apiClient.sendMessage(
        _conversationId,
        content,
        const Uuid().v4(),
      );

      final confirmed = _messageFromJson(serverMessage, fallback: optimistic)
          .copyWithMe(isMe: true, isAgent: false);
      state = [
        for (final msg in state)
          if (msg.id == optimistic.id) confirmed else msg,
      ];
      ref.read(conversationsNotifierProvider.notifier).upsertFromMessage(confirmed);
    } catch (_) {
      // Keep optimistic message and wait for eventual WS sync.
    }
  }

  Message _messageFromJson(
    Map<String, dynamic> json, {
    Message? fallback,
  }) {
    return Message(
      id: (json['id'] ?? fallback?.id ?? const Uuid().v4()).toString(),
      conversationId:
          (json['conversation_id'] ?? fallback?.conversationId ?? _conversationId)
              .toString(),
      senderId: (json['sender_id'] ?? fallback?.senderId ?? 'unknown').toString(),
      content: (json['content'] ?? fallback?.content ?? '').toString(),
      format: _toMessageFormat((json['format'] ?? 'plain').toString()),
      seq: (json['seq'] as num?)?.toInt() ?? fallback?.seq ?? 0,
      createdAt: DateTime.tryParse((json['created_at'] ?? '').toString()) ??
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
    createdAt: DateTime.tryParse((json['created_at'] ?? '').toString()) ??
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
    );
  }
}
