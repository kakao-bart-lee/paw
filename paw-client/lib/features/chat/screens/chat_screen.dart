import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
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
    await ref
        .read(messagesNotifierProvider(widget.conversationId).notifier)
        .sendMessage(text);

    // Scroll to bottom after a short delay to allow the list to update
    Future.delayed(const Duration(milliseconds: 50), _scrollToBottom);
  }

  @override
  Widget build(BuildContext context) {
    final messages = ref.watch(messagesNotifierProvider(widget.conversationId));
    final activeStreams = ref.watch(messagesNotifierProvider(widget.conversationId).notifier).activeStreams;
    
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
                color: conversation.isE2ee ? const Color(0xFF4CAF50) : Colors.grey,
              ),
            ),
          ],
        ),
        actions: [
          if (conversation.agents.isEmpty)
            IconButton(
              icon: const Icon(Icons.group),
              onPressed: () => context.push('/group/${widget.conversationId}/info'),
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
                // Stub for activating E2EE
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('E2EE 활성화 요청됨')),
                );
              },
            ),
          Expanded(
            child: ListView.builder(
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
          ),
          Consumer(
            builder: (context, ref, _) {
              final typing = ref.watch(typingProvider);
              final typingInConv = typing[widget.conversationId] ?? {};
              if (typingInConv.isEmpty) return const SizedBox.shrink();
              return const TypingIndicator(userName: '상대방');
            },
          ),
          MessageInput(
            onSend: _handleSend,
          ),
        ],
      ),
    );
  }
}
