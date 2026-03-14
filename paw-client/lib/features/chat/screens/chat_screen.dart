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
    final titleTheme = Theme.of(context).textTheme;

    return Scaffold(
      backgroundColor: AppTheme.surface1,
      appBar: AppBar(
        leadingWidth: !isDesktop && Navigator.canPop(context) ? 44 : null,
        leading: !isDesktop && Navigator.canPop(context)
            ? IconButton(
                icon: const Icon(Icons.chevron_left_rounded),
                onPressed: () => context.pop(),
              )
            : null,
        automaticallyImplyLeading: false,
        titleSpacing: !isDesktop ? 0 : 16,
        bottom: PreferredSize(
          preferredSize: const Size.fromHeight(1),
          child: Container(
            height: 1,
            color: AppTheme.outline.withValues(alpha: 0.72),
          ),
        ),
        title: Row(
          children: [
            MessengerAvatar(
              name: conversation.name,
              imageUrl: conversation.avatarUrl,
              size: 38,
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
                          style: titleTheme.titleMedium?.copyWith(
                            fontWeight: FontWeight.w700,
                          ),
                        ),
                      ),
                      if (conversation.isE2ee) ...[
                        const SizedBox(width: 6),
                        Icon(
                          Icons.lock_rounded,
                          size: 15,
                          color: AppTheme.accent.withValues(alpha: 0.88),
                        ),
                      ],
                    ],
                  ),
                  const SizedBox(height: 3),
                  Text(
                    conversation.agents.isNotEmpty
                        ? 'AI Agent · 항상 응답 가능'
                        : conversation.isE2ee
                        ? '보안 대화 · 종단간 암호화'
                        : '대화 중',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: titleTheme.bodySmall,
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
            colors: [
              AppTheme.agentBubbleDark,
              AppTheme.surface1,
              AppTheme.background,
            ],
            stops: [0.0, 0.32, 1.0],
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
                padding: const EdgeInsets.fromLTRB(12, 8, 12, 0),
                child: Container(
                  width: double.infinity,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 14,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    color: AppTheme.webNoticeSurface,
                    borderRadius: BorderRadius.circular(AppTheme.radiusMd),
                    border: Border.all(
                      color: AppTheme.warning.withValues(alpha: 0.22),
                    ),
                  ),
                  child: Text(
                    '웹에서는 E2EE/Rust 기능을 지원하지 않습니다.',
                    textAlign: TextAlign.center,
                    style: titleTheme.labelMedium?.copyWith(
                      color: AppTheme.warning,
                    ),
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
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: Text(messagesError ?? '메시지를 불러오지 못했습니다.'),
                  ),
                ),
                ResourceLoadState.ready =>
                  itemCount == 0
                      ? _EmptyChatState(conversation: conversation)
                      : ListView.builder(
                          controller: _scrollController,
                          reverse: true,
                          padding: EdgeInsets.fromLTRB(
                            0,
                            18,
                            0,
                            isDesktop ? 20 : 12,
                          ),
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
                if (typingInConversation.isEmpty) {
                  return const SizedBox.shrink();
                }
                return const Padding(
                  padding: EdgeInsets.only(left: 12, right: 12, bottom: 6),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: TypingIndicator(userName: '상대방'),
                  ),
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

class _EmptyChatState extends StatelessWidget {
  const _EmptyChatState({required this.conversation});

  final Conversation conversation;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(28),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            MessengerAvatar(
              name: conversation.name,
              imageUrl: conversation.avatarUrl,
              size: 72,
              isAgent: conversation.agents.isNotEmpty,
              showPresence: false,
            ),
            const SizedBox(height: 16),
            Text('메시지가 없습니다.', style: Theme.of(context).textTheme.titleLarge),
            const SizedBox(height: 8),
            Text(
              conversation.agents.isNotEmpty
                  ? '${conversation.name}에서 질문을 남기면 AI가 같은 톤으로 이어서 답변합니다.'
                  : '${conversation.name}에서 첫 메시지로 대화를 시작해보세요.',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall,
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

    final (text, background, foreground) = switch (state) {
      WsConnectionState.connecting => (
        '서버에 연결 중입니다...',
        AppTheme.infoSurface,
        AppTheme.info,
      ),
      WsConnectionState.retrying => (
        '연결이 끊겨 재시도 중입니다...',
        AppTheme.warningSurface,
        AppTheme.warning,
      ),
      WsConnectionState.disconnected => (
        '오프라인 상태입니다. 네트워크를 확인해주세요.',
        AppTheme.dangerSurface,
        AppTheme.danger.withValues(alpha: 0.82),
      ),
      WsConnectionState.connected => (
        '',
        Colors.transparent,
        Colors.transparent,
      ),
    };

    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 8, 12, 0),
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
        decoration: BoxDecoration(
          color: background,
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          border: Border.all(color: foreground.withValues(alpha: 0.18)),
        ),
        child: Text(
          text,
          textAlign: TextAlign.center,
          style: Theme.of(
            context,
          ).textTheme.labelMedium?.copyWith(color: foreground),
        ),
      ),
    );
  }
}
