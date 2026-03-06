import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:uuid/uuid.dart';
import '../models/message.dart';
import '../providers/chat_provider.dart';
import '../widgets/message_bubble.dart';
import '../widgets/message_input.dart';

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

  void _handleSend(String text) {
    final messages = ref.read(messagesNotifierProvider(widget.conversationId));
    final seq = messages.isNotEmpty ? messages.last.seq + 1 : 1;
    
    final newMessage = Message(
      id: const Uuid().v4(),
      conversationId: widget.conversationId,
      senderId: 'me',
      content: text,
      format: MessageFormat.plain,
      seq: seq,
      createdAt: DateTime.now(),
      isMe: true,
      isAgent: false,
    );

    ref.read(messagesNotifierProvider(widget.conversationId).notifier).addMessage(newMessage);
    
    // Scroll to bottom after a short delay to allow the list to update
    Future.delayed(const Duration(milliseconds: 50), _scrollToBottom);
  }

  @override
  Widget build(BuildContext context) {
    final messages = ref.watch(messagesNotifierProvider(widget.conversationId));
    // Reverse the list for ListView.builder with reverse: true
    final reversedMessages = messages.reversed.toList();

    return Scaffold(
      appBar: AppBar(
        title: const Text('대화'),
      ),
      body: Column(
        children: [
          Expanded(
            child: ListView.builder(
              controller: _scrollController,
              reverse: true,
              padding: const EdgeInsets.symmetric(vertical: 16),
              itemCount: reversedMessages.length,
              itemBuilder: (context, index) {
                final message = reversedMessages[index];
                return MessageBubble(message: message);
              },
            ),
          ),
          MessageInput(
            onSend: _handleSend,
          ),
        ],
      ),
    );
  }
}
