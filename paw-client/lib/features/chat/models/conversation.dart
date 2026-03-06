import 'message.dart';

class Conversation {
  final String id;
  final String name;
  final String? avatarUrl;
  final Message? lastMessage;
  final int unreadCount;
  final DateTime updatedAt;
  
  const Conversation({
    required this.id,
    required this.name,
    this.avatarUrl,
    this.lastMessage,
    required this.unreadCount,
    required this.updatedAt,
  });
}
