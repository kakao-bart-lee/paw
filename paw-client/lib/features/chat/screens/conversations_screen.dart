import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';
import '../models/conversation.dart';
import '../providers/chat_provider.dart';
import '../widgets/conversation_tile.dart';
import 'chat_screen.dart';

const double kDesktopBreakpoint = 960;

enum _ConversationFilter { all, secure, agents, unread }

class ConversationsScreen extends ConsumerStatefulWidget {
  const ConversationsScreen({super.key});

  @override
  ConsumerState<ConversationsScreen> createState() =>
      _ConversationsScreenState();
}

class _ConversationsScreenState extends ConsumerState<ConversationsScreen> {
  String? _selectedConversationId;
  _ConversationFilter _filter = _ConversationFilter.all;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(conversationsNotifierProvider.notifier).refresh();
    });
  }

  @override
  Widget build(BuildContext context) {
    final conversations = ref.watch(conversationsNotifierProvider);
    final loadState = ref.watch(conversationsLoadStateProvider);
    final loadError = ref.watch(conversationsErrorProvider);

    return LayoutBuilder(
      builder: (context, constraints) {
        final isWide = constraints.maxWidth >= kDesktopBreakpoint;
        final filteredConversations = _applyFilter(conversations);

        if (isWide) {
          return _buildDesktopLayout(
            context,
            filteredConversations,
            loadState,
            loadError,
          );
        }

        return _buildMobileLayout(
          context,
          filteredConversations,
          loadState,
          loadError,
        );
      },
    );
  }

  List<Conversation> _applyFilter(List<Conversation> conversations) {
    return conversations.where((conversation) {
      switch (_filter) {
        case _ConversationFilter.all:
          return true;
        case _ConversationFilter.secure:
          return conversation.isE2ee;
        case _ConversationFilter.agents:
          return conversation.agents.isNotEmpty;
        case _ConversationFilter.unread:
          return conversation.unreadCount > 0;
      }
    }).toList();
  }

  Widget _buildMobileLayout(
    BuildContext context,
    List<Conversation> conversations,
    ResourceLoadState loadState,
    String? loadError,
  ) {
    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('채팅', style: Theme.of(context).textTheme.titleLarge),
            Text(
              'AI와 팀 메시지를 한곳에서',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.search_rounded),
            onPressed: () => context.push('/search'),
            tooltip: '검색',
          ),
          IconButton(
            icon: const Icon(Icons.edit_square),
            onPressed: () => context.push('/create-group'),
            tooltip: '새 대화',
          ),
        ],
      ),
      body: Column(
        children: [
          _ConversationHeader(
            filter: _filter,
            onFilterChanged: (filter) => setState(() => _filter = filter),
            onSearchTap: () => context.push('/search'),
          ),
          Expanded(
            child: _buildConversationList(
              context,
              conversations,
              loadState,
              loadError,
            ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => context.push('/create-group'),
        backgroundColor: AppTheme.primary,
        foregroundColor: AppTheme.background,
        icon: const Icon(Icons.add_rounded),
        label: const Text('새 대화'),
      ),
    );
  }

  Widget _buildDesktopLayout(
    BuildContext context,
    List<Conversation> conversations,
    ResourceLoadState loadState,
    String? loadError,
  ) {
    final selectedConversation = _selectedConversationId;

    return Scaffold(
      backgroundColor: Colors.transparent,
      body: Padding(
        padding: const EdgeInsets.fromLTRB(18, 16, 16, 16),
        child: Row(
          children: [
            Container(
              width: 360,
              decoration: BoxDecoration(
                color: AppTheme.surface2,
                borderRadius: BorderRadius.circular(30),
                border: Border.all(color: AppTheme.outline),
              ),
              child: Column(
                children: [
                  Padding(
                    padding: const EdgeInsets.fromLTRB(20, 20, 20, 8),
                    child: Row(
                      children: [
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                '채팅',
                                style: Theme.of(
                                  context,
                                ).textTheme.headlineMedium,
                              ),
                              const SizedBox(height: 4),
                              Text(
                                '대화, 협업, 에이전트를 매끄럽게 넘나드는 받은편지함',
                                style: Theme.of(context).textTheme.bodySmall,
                              ),
                            ],
                          ),
                        ),
                        IconButton(
                          icon: const Icon(Icons.search_rounded),
                          onPressed: () => context.push('/search'),
                          tooltip: '검색',
                        ),
                        IconButton(
                          icon: const Icon(Icons.edit_square),
                          onPressed: () => context.push('/create-group'),
                          tooltip: '새 대화',
                        ),
                      ],
                    ),
                  ),
                  _ConversationHeader(
                    filter: _filter,
                    onFilterChanged: (filter) =>
                        setState(() => _filter = filter),
                    onSearchTap: () => context.push('/search'),
                    compact: true,
                  ),
                  Expanded(
                    child: _buildConversationList(
                      context,
                      conversations,
                      loadState,
                      loadError,
                      desktop: true,
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: Container(
                decoration: BoxDecoration(
                  color: AppTheme.surface1,
                  borderRadius: BorderRadius.circular(32),
                  border: Border.all(color: AppTheme.outline),
                ),
                clipBehavior: Clip.antiAlias,
                child: selectedConversation != null
                    ? ChatScreen(conversationId: selectedConversation)
                    : _EmptyDesktopPanel(count: conversations.length),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildConversationList(
    BuildContext context,
    List<Conversation> conversations,
    ResourceLoadState loadState,
    String? loadError, {
    bool desktop = false,
  }) {
    if (loadState == ResourceLoadState.loading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (loadState == ResourceLoadState.error) {
      return Center(child: Text(loadError ?? '대화 목록을 불러오지 못했습니다.'));
    }

    if (conversations.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Container(
                width: 72,
                height: 72,
                decoration: BoxDecoration(
                  color: AppTheme.surface3,
                  borderRadius: BorderRadius.circular(24),
                  border: Border.all(color: AppTheme.outline),
                ),
                child: const Icon(Icons.chat_bubble_outline_rounded, size: 30),
              ),
              const SizedBox(height: 18),
              Text(
                '아직 대화가 없습니다',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              Text(
                '새 대화를 시작하거나 검색으로 메시지를 빠르게 찾아보세요.',
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ),
        ),
      );
    }

    return ListView.builder(
      padding: EdgeInsets.only(bottom: desktop ? 12 : 96),
      itemCount: conversations.length,
      itemBuilder: (context, index) {
        final conversation = conversations[index];
        return ConversationTile(
          conversation: conversation,
          selected: desktop && _selectedConversationId == conversation.id,
          onTap: () {
            if (desktop) {
              setState(() {
                _selectedConversationId = conversation.id;
              });
            } else {
              context.push('/chat/${conversation.id}');
            }
          },
        );
      },
    );
  }
}

class _ConversationHeader extends StatelessWidget {
  const _ConversationHeader({
    required this.filter,
    required this.onFilterChanged,
    required this.onSearchTap,
    this.compact = false,
  });

  final _ConversationFilter filter;
  final ValueChanged<_ConversationFilter> onFilterChanged;
  final VoidCallback onSearchTap;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.fromLTRB(
        compact ? 16 : 16,
        compact ? 8 : 4,
        compact ? 16 : 16,
        12,
      ),
      child: Column(
        children: [
          InkWell(
            borderRadius: BorderRadius.circular(22),
            onTap: onSearchTap,
            child: Ink(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
              decoration: BoxDecoration(
                color: AppTheme.surface2,
                borderRadius: BorderRadius.circular(22),
                border: Border.all(color: AppTheme.outline),
              ),
              child: Row(
                children: [
                  const Icon(Icons.search_rounded, color: AppTheme.mutedText),
                  const SizedBox(width: 12),
                  Text(
                    '메시지 검색',
                    style: Theme.of(
                      context,
                    ).textTheme.bodyMedium?.copyWith(color: AppTheme.mutedText),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
          SizedBox(
            height: 34,
            child: ListView(
              scrollDirection: Axis.horizontal,
              children: [
                _FilterChipButton(
                  label: '전체',
                  selected: filter == _ConversationFilter.all,
                  onTap: () => onFilterChanged(_ConversationFilter.all),
                ),
                _FilterChipButton(
                  label: '보안',
                  selected: filter == _ConversationFilter.secure,
                  onTap: () => onFilterChanged(_ConversationFilter.secure),
                ),
                _FilterChipButton(
                  label: 'Agent',
                  selected: filter == _ConversationFilter.agents,
                  onTap: () => onFilterChanged(_ConversationFilter.agents),
                ),
                _FilterChipButton(
                  label: '안 읽음',
                  selected: filter == _ConversationFilter.unread,
                  onTap: () => onFilterChanged(_ConversationFilter.unread),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _FilterChipButton extends StatelessWidget {
  const _FilterChipButton({
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(right: 8),
      child: InkWell(
        borderRadius: BorderRadius.circular(999),
        onTap: onTap,
        child: Ink(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
          decoration: BoxDecoration(
            color: selected ? AppTheme.primarySoft : AppTheme.surface3,
            borderRadius: BorderRadius.circular(999),
            border: Border.all(
              color: selected
                  ? AppTheme.primary.withValues(alpha: 0.28)
                  : AppTheme.outline,
            ),
          ),
          child: Text(
            label,
            style: Theme.of(context).textTheme.labelMedium?.copyWith(
              color: selected ? AppTheme.primary : AppTheme.mutedText,
              fontWeight: FontWeight.w700,
            ),
          ),
        ),
      ),
    );
  }
}

class _EmptyDesktopPanel extends StatelessWidget {
  const _EmptyDesktopPanel({required this.count});

  final int count;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 84,
              height: 84,
              decoration: BoxDecoration(
                color: AppTheme.surface2,
                borderRadius: BorderRadius.circular(28),
                border: Border.all(color: AppTheme.outline),
              ),
              child: const Icon(
                Icons.forum_rounded,
                size: 34,
                color: AppTheme.primary,
              ),
            ),
            const SizedBox(height: 20),
            Text(
              '대화를 선택하세요',
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            const SizedBox(height: 10),
            Text(
              '$count개의 대화가 준비되어 있습니다. 왼쪽 목록에서 하나를 선택하면 메시지 흐름이 여기 표시됩니다.',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}
