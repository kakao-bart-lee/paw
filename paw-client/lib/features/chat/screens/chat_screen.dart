import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/di/service_locator.dart';
import '../../../core/http/api_client.dart';
import '../../../core/theme/app_theme.dart';
import '../../../core/widgets/messenger_avatar.dart';
import '../../../core/ws/ws_service.dart';
import '../models/conversation.dart';
import '../providers/chat_provider.dart';
import '../providers/typing_provider.dart';
import '../widgets/agent_consent_banner.dart';
import '../widgets/e2ee_banner.dart';
import '../widgets/message_bubble.dart';
import '../widgets/message_input.dart';
import '../widgets/stream_bubble.dart';
import '../widgets/typing_indicator.dart';

class ChatScreen extends ConsumerStatefulWidget {
  const ChatScreen({super.key, required this.conversationId});

  final String conversationId;

  @override
  ConsumerState<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends ConsumerState<ChatScreen> {
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _scrollToBottom() {
    if (!_scrollController.hasClients) return;
    _scrollController.animateTo(
      0.0,
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeOut,
    );
  }

  Future<void> _handleSend(String text) async {
    final result = await ref
        .read(messagesNotifierProvider(widget.conversationId).notifier)
        .sendMessage(text);
    if (!mounted) return;

    if (!result.ok) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(result.message ?? '메시지 전송에 실패했습니다.')),
      );
      return;
    }

    Future<void>.delayed(const Duration(milliseconds: 50), _scrollToBottom);
  }

  @override
  Widget build(BuildContext context) {
    final messages = ref.watch(messagesNotifierProvider(widget.conversationId));
    final messagesLoadState = ref.watch(
      messagesLoadStateProvider(widget.conversationId),
    );
    final messagesError = ref.watch(
      messagesErrorProvider(widget.conversationId),
    );
    final wsService = getIt.isRegistered<WsService>()
        ? getIt<WsService>()
        : null;
    final apiClient = getIt.isRegistered<ApiClient>()
        ? getIt<ApiClient>()
        : null;
    final hasToken = apiClient?.accessToken?.isNotEmpty ?? false;
    final activeStreams = ref
        .watch(messagesNotifierProvider(widget.conversationId).notifier)
        .activeStreams;

    final reversedMessages = messages.reversed.toList();
    final activeStreamsList = activeStreams.values
        .where((stream) => stream.conversationId == widget.conversationId)
        .toList()
        .reversed
        .toList();
    final itemCount = reversedMessages.length + activeStreamsList.length;

    final conversations = ref.watch(conversationsNotifierProvider);
    final conversation = conversations.firstWhere(
      (c) => c.id == widget.conversationId,
      orElse: () => Conversation(
        id: widget.conversationId,
        name: '대화',
        unreadCount: 0,
        updatedAt: DateTime.now(),
      ),
    );
    final isDesktop = MediaQuery.sizeOf(context).width >= 960;

    return Scaffold(
      backgroundColor: AppTheme.surface1,
      appBar: AppBar(
        leading: !isDesktop && Navigator.canPop(context)
            ? IconButton(
                icon: const Icon(Icons.chevron_left_rounded),
                onPressed: () => context.pop(),
              )
            : null,
        automaticallyImplyLeading: false,
        titleSpacing: !isDesktop ? 0 : 16,
        title: Row(
          children: [
            MessengerAvatar(
              name: conversation.name,
              imageUrl: conversation.avatarUrl,
              size: 40,
              isAgent: conversation.agents.isNotEmpty,
              isOnline:
                  activeStreamsList.isNotEmpty || conversation.unreadCount > 0,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Row(
                    children: [
                      Flexible(
                        child: Text(
                          conversation.name,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: Theme.of(context).textTheme.titleMedium
                              ?.copyWith(fontWeight: FontWeight.w700),
                        ),
                      ),
                      if (conversation.isE2ee) ...[
                        const SizedBox(width: 6),
                        Icon(
                          Icons.lock_rounded,
                          size: 15,
                          color: AppTheme.primary.withValues(alpha: 0.8),
                        ),
                      ],
                    ],
                  ),
                  const SizedBox(height: 2),
                  Text(
                    conversation.agents.isNotEmpty
                        ? 'AI Agent · 항상 응답 가능'
                        : conversation.isE2ee
                        ? '보안 대화 · 종단간 암호화'
                        : '대화 중',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
          ],
        ),
        actions: [
          if (conversation.agents.isEmpty)
            IconButton(
              icon: const Icon(Icons.call_outlined),
              onPressed: () {},
              tooltip: '통화',
            ),
          IconButton(
            icon: const Icon(Icons.info_outline_rounded),
            onPressed: () {
              if (conversation.agents.isEmpty) {
                context.push('/group/${widget.conversationId}/info');
              } else {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('에이전트 상세 정보는 곧 제공됩니다.')),
                );
              }
            },
            tooltip: '정보',
          ),
        ],
      ),
      body: DecoratedBox(
        decoration: const BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [Color(0xFF111A1E), AppTheme.surface1],
          ),
        ),
        child: Column(
          children: [
            if (conversation.agents.isNotEmpty)
              AgentConsentBanner(
                agentNames: conversation.agents,
                conversationId: widget.conversationId,
              )
            else if (conversation.isE2ee)
              const E2eeBanner(type: E2eeBannerType.active)
            else
              E2eeBanner(
                type: E2eeBannerType.available,
                onActivate: () {
                  if (kIsWeb) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('웹에서는 E2EE/Rust 기능이 지원되지 않습니다.'),
                      ),
                    );
                    return;
                  }
                  ScaffoldMessenger.of(
                    context,
                  ).showSnackBar(const SnackBar(content: Text('E2EE 활성화 요청됨')));
                },
              ),
            if (kIsWeb && conversation.isE2ee)
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 10, 16, 0),
                child: Container(
                  width: double.infinity,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 14,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    color: const Color(0xFF2B2416),
                    borderRadius: BorderRadius.circular(16),
                    border: Border.all(
                      color: AppTheme.warning.withValues(alpha: 0.24),
                    ),
                  ),
                  child: Text(
                    '웹에서는 E2EE/Rust 기능을 지원하지 않습니다.',
                    textAlign: TextAlign.center,
                    style: Theme.of(
                      context,
                    ).textTheme.labelMedium?.copyWith(color: AppTheme.warning),
                  ),
                ),
              ),
            if (wsService != null)
              ValueListenableBuilder<WsConnectionState>(
                valueListenable: wsService.connectionState,
                builder: (context, state, _) => _WsStatusBanner(state: state),
              )
            else
              const _WsStatusBanner(state: WsConnectionState.disconnected),
            Expanded(
              child: switch (messagesLoadState) {
                ResourceLoadState.loading => const Center(
                  child: CircularProgressIndicator(),
                ),
                ResourceLoadState.error => Center(
                  child: Text(messagesError ?? '메시지를 불러오지 못했습니다.'),
                ),
                ResourceLoadState.ready =>
                  itemCount == 0
                      ? const Center(child: Text('메시지가 없습니다.'))
                      : ListView.builder(
                          controller: _scrollController,
                          reverse: true,
                          padding: const EdgeInsets.symmetric(vertical: 16),
                          itemCount: itemCount,
                          itemBuilder: (context, index) {
                            if (index < activeStreamsList.length) {
                              final stream = activeStreamsList[index];
                              return StreamBubble(
                                streamId: stream.streamId,
                                contentNotifier: stream.contentNotifier,
                                isComplete: stream.isComplete,
                                toolName: stream.currentTool,
                                toolLabel: stream.currentToolLabel,
                                toolComplete: stream.toolComplete,
                              );
                            }

                            final messageIndex =
                                index - activeStreamsList.length;
                            final message = reversedMessages[messageIndex];
                            return MessageBubble(message: message);
                          },
                        ),
              },
            ),
            Consumer(
              builder: (context, ref, _) {
                final typing = ref.watch(typingProvider);
                final typingInConversation =
                    typing[widget.conversationId] ?? {};
                if (typingInConversation.isEmpty)
                  return const SizedBox.shrink();
                return const Padding(
                  padding: EdgeInsets.only(left: 16, right: 16, bottom: 6),
                  child: TypingIndicator(userName: '상대방'),
                );
              },
            ),
            if (wsService != null)
              ValueListenableBuilder<WsConnectionState>(
                valueListenable: wsService.connectionState,
                builder: (context, wsState, _) {
                  final canSend =
                      hasToken && wsState == WsConnectionState.connected;
                  return MessageInput(
                    onSend: _handleSend,
                    canSend: canSend,
                    sendDisabledReason: hasToken
                        ? switch (wsState) {
                            WsConnectionState.connecting =>
                              '연결 중입니다. 잠시만 기다려주세요.',
                            WsConnectionState.retrying =>
                              '재연결 중입니다. 연결이 복구되면 전송할 수 있습니다.',
                            WsConnectionState.disconnected =>
                              '연결이 끊겼습니다. 재시도 중입니다.',
                            WsConnectionState.connected => null,
                          }
                        : '로그인 상태가 만료되어 전송할 수 없습니다.',
                  );
                },
              )
            else
              MessageInput(
                onSend: _handleSend,
                canSend: false,
                sendDisabledReason: '연결 서비스가 준비되지 않았습니다.',
              ),
          ],
        ),
      ),
    );
  }
}

class _WsStatusBanner extends StatelessWidget {
  const _WsStatusBanner({required this.state});

  final WsConnectionState state;

  @override
  Widget build(BuildContext context) {
    if (state == WsConnectionState.connected) {
      return const SizedBox.shrink();
    }

    final (text, color) = switch (state) {
      WsConnectionState.connecting => (
        '서버에 연결 중입니다...',
        const Color(0xFF1C4F7A),
      ),
      WsConnectionState.retrying => (
        '연결이 끊겨 재시도 중입니다...',
        const Color(0xFF6C4B16),
      ),
      WsConnectionState.disconnected => (
        '오프라인 상태입니다. 네트워크를 확인해주세요.',
        const Color(0xFF5C1F28),
      ),
      WsConnectionState.connected => ('', Colors.transparent),
    };

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 10, 16, 0),
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
        decoration: BoxDecoration(
          color: color,
          borderRadius: BorderRadius.circular(16),
        ),
        child: Text(
          text,
          textAlign: TextAlign.center,
          style: Theme.of(
            context,
          ).textTheme.labelMedium?.copyWith(color: Colors.white),
        ),
      ),
    );
  }
}
