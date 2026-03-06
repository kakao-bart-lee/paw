import 'package:riverpod_annotation/riverpod_annotation.dart';
import '../models/conversation.dart';
import '../models/message.dart';

part 'chat_provider.g.dart';

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

@riverpod
class ConversationsNotifier extends _$ConversationsNotifier {
  @override
  List<Conversation> build() => _mockConversations;
}

@riverpod
class MessagesNotifier extends _$MessagesNotifier {
  @override
  List<Message> build(String conversationId) {
    return _mockMessagesData[conversationId] ?? [];
  }
  
  void addMessage(Message msg) {
    state = [...state, msg];
  }
}
