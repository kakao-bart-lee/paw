enum MessageFormat { markdown, plain }
enum MessageSender { me, other, agent }

class Message {
  final String id;
  final String conversationId;
  final String senderId;
  final String content;
  final MessageFormat format;
  final int seq;
  final DateTime createdAt;
  final bool isMe;
  final bool isAgent;
  
  const Message({
    required this.id,
    required this.conversationId,
    required this.senderId,
    required this.content,
    required this.format,
    required this.seq,
    required this.createdAt,
    required this.isMe,
    required this.isAgent,
  });
}
