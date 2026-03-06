import 'message.dart';

class Conversation {
  final String id;
  final String name;
  final String? avatarUrl;
  final Message? lastMessage;
  final int unreadCount;
  final DateTime updatedAt;
  final bool isE2ee;
  final List<String> agents;
  
  const Conversation({
    required this.id,
    required this.name,
    this.avatarUrl,
    this.lastMessage,
    required this.unreadCount,
    required this.updatedAt,
    this.isE2ee = false,
    this.agents = const [],
  });
}
