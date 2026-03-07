import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../../core/di/service_locator.dart';
import '../../../core/http/api_client.dart';
import '../../../core/ws/ws_service.dart';
import '../providers/chat_provider.dart';
import '../providers/typing_provider.dart';
import '../widgets/message_bubble.dart';
import '../widgets/stream_bubble.dart';
import '../widgets/message_input.dart';
import '../widgets/typing_indicator.dart';
import '../widgets/e2ee_banner.dart';
import '../widgets/agent_consent_banner.dart';
import '../models/conversation.dart';

class ChatScreen extends ConsumerStatefulWidget {
  final String conversationId;
  const ChatScreen({super.key, required this.conversationId});

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
    if (_scrollController.hasClients) {
      _scrollController.animateTo(
        0.0,
        duration: const Duration(milliseconds: 300),
        curve: Curves.easeOut,
      );
    }
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

    // Scroll to bottom after a short delay to allow the list to update
    Future.delayed(const Duration(milliseconds: 50), _scrollToBottom);
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

    // Sort by sequence/time? Active streams are always at the bottom.
    // Since we reverse the list, active streams should be at the beginning of the reversed list.
    final reversedMessages = messages.reversed.toList();
    final activeStreamsList = activeStreams.values
        .where((s) => s.conversationId == widget.conversationId)
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

    return Scaffold(
      appBar: AppBar(
        title: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(conversation.name),
            const SizedBox(width: 8),
            GestureDetector(
              onTap: () {
                context.push('/chat/${widget.conversationId}/verify');
              },
              child: Icon(
                conversation.isE2ee ? Icons.lock : Icons.lock_open,
                size: 16,
                color: conversation.isE2ee
                    ? const Color(0xFF4CAF50)
                    : Colors.grey,
              ),
            ),
          ],
        ),
        actions: [
          if (conversation.agents.isEmpty)
            IconButton(
              icon: const Icon(Icons.group),
              onPressed: () =>
                  context.push('/group/${widget.conversationId}/info'),
            ),
        ],
      ),
      body: Column(
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
            Container(
              width: double.infinity,
              color: const Color(0xFF2E1B1B),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              child: const Text(
                '웹에서는 E2EE/Rust 기능을 지원하지 않습니다.',
                style: TextStyle(color: Color(0xFFFFC107), fontSize: 12),
                textAlign: TextAlign.center,
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

                          final messageIndex = index - activeStreamsList.length;
                          final message = reversedMessages[messageIndex];
                          return MessageBubble(message: message);
                        },
                      ),
            },
          ),
          Consumer(
            builder: (context, ref, _) {
              final typing = ref.watch(typingProvider);
              final typingInConv = typing[widget.conversationId] ?? {};
              if (typingInConv.isEmpty) return const SizedBox.shrink();
              return const TypingIndicator(userName: '상대방');
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
        const Color(0xFF0D47A1),
      ),
      WsConnectionState.retrying => (
        '연결이 끊겨 재시도 중입니다...',
        const Color(0xFFEF6C00),
      ),
      WsConnectionState.disconnected => (
        '오프라인 상태입니다. 네트워크를 확인해주세요.',
        const Color(0xFFB71C1C),
      ),
      WsConnectionState.connected => ('', Colors.transparent),
    };

    return Container(
      width: double.infinity,
      color: color,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      child: Text(
        text,
        style: const TextStyle(color: Colors.white, fontSize: 12),
        textAlign: TextAlign.center,
      ),
    );
  }
}
