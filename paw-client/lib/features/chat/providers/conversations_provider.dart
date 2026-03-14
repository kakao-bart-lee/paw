import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/di/service_locator.dart';
import '../../../core/errors/app_error.dart';
import '../../../core/http/api_client.dart';
import '../../auth/providers/auth_provider.dart';
import '../models/conversation.dart';
import '../models/message.dart';
import 'chat_types.dart';

final conversationsNotifierProvider =
    NotifierProvider<ConversationsNotifier, List<Conversation>>(
      ConversationsNotifier.new,
    );

class ConversationsNotifier extends Notifier<List<Conversation>> {
  ApiClient? get _apiClient =>
      getIt.isRegistered<ApiClient>() ? getIt<ApiClient>() : null;

  @override
  List<Conversation> build() {
    final auth = ref.watch(
      authNotifierProvider.select(
        (state) => (state.step, state.isLoading, state.accessToken),
      ),
    );

    Future.microtask(() async {
      final (step, isLoading, accessToken) = auth;
      if (step == AuthStep.authenticated &&
          !isLoading &&
          accessToken != null &&
          accessToken.isNotEmpty) {
        await _loadConversations();
        return;
      }

      ref.read(conversationsErrorProvider.notifier).state = null;
      ref.read(conversationsLoadStateProvider.notifier).state = isLoading
          ? ResourceLoadState.loading
          : ResourceLoadState.ready;
      state = const [];
    });

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
          ? messageFromJson(lastMessage)
          : null,
      isE2ee: json['is_e2ee'] == true,
      agents:
          (json['agents'] as List?)?.map((e) => e.toString()).toList() ??
          const [],
    );
  }
}
