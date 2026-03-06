import 'dart:async';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'typing_provider.g.dart';

@riverpod
class TypingNotifier extends _$TypingNotifier {
  final Map<String, Map<String, Timer>> _timers = {};

  static const _autoExpire = Duration(seconds: 5);

  @override
  Map<String, Set<String>> build() => {};

  void setTyping(String conversationId, String userId, bool isTyping) {
    final current = Map<String, Set<String>>.from(state);
    final users = Set<String>.from(current[conversationId] ?? {});

    _timers[conversationId]?[userId]?.cancel();

    if (isTyping) {
      users.add(userId);
      _timers.putIfAbsent(conversationId, () => {});
      _timers[conversationId]![userId] = Timer(_autoExpire, () {
        setTyping(conversationId, userId, false);
      });
    } else {
      users.remove(userId);
      _timers[conversationId]?.remove(userId);
    }

    if (users.isEmpty) {
      current.remove(conversationId);
    } else {
      current[conversationId] = users;
    }

    state = current;
  }
}
