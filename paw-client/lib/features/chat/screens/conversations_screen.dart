import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/chat_provider.dart';
import '../widgets/conversation_tile.dart';
import 'chat_screen.dart';

/// Breakpoint (in logical pixels) above which the two-panel desktop
/// layout is used instead of the single-panel mobile layout.
const double kDesktopBreakpoint = 768;

class ConversationsScreen extends ConsumerStatefulWidget {
  const ConversationsScreen({super.key});

  @override
  ConsumerState<ConversationsScreen> createState() =>
      _ConversationsScreenState();
}

class _ConversationsScreenState extends ConsumerState<ConversationsScreen> {
  String? _selectedConversationId;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      unawaited(ref.read(conversationsNotifierProvider.notifier).refresh());
      Future<void>.delayed(const Duration(milliseconds: 750), () async {
        if (!mounted) {
          return;
        }

        final loadState = ref.read(conversationsLoadStateProvider);
        final conversations = ref.read(conversationsNotifierProvider);
        if (loadState == ResourceLoadState.ready && conversations.isEmpty) {
          await ref.read(conversationsNotifierProvider.notifier).refresh();
        }
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    final conversations = ref.watch(conversationsNotifierProvider);
    final loadState = ref.watch(conversationsLoadStateProvider);
    final loadError = ref.watch(conversationsErrorProvider);

    return LayoutBuilder(
      builder: (context, constraints) {
        final isWide = constraints.maxWidth > kDesktopBreakpoint;

        if (isWide) {
          return _buildDesktopLayout(
            context,
            conversations,
            loadState,
            loadError,
          );
        }
        return _buildMobileLayout(context, conversations, loadState, loadError);
      },
    );
  }

  // ── Mobile: single-panel (existing behaviour) ──────────────────────

  Widget _buildMobileLayout(
    BuildContext context,
    List conversations,
    ResourceLoadState loadState,
    String? loadError,
  ) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('\ucc44\ud305'),
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () => context.push('/search'),
            tooltip: '\uac80\uc0c9',
          ),
          IconButton(
            icon: const Icon(Icons.edit_square),
            onPressed: () {},
            tooltip: '\uc0c8 \ub300\ud654',
          ),
        ],
      ),
      body: _buildConversationList(conversations, loadState, loadError),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () {},
        icon: const Icon(Icons.add),
        label: const Text('\uc0c8 \ub300\ud654'),
      ),
    );
  }

  // ── Desktop: two-panel layout ──────────────────────────────────────

  Widget _buildDesktopLayout(
    BuildContext context,
    List conversations,
    ResourceLoadState loadState,
    String? loadError,
  ) {
    return Scaffold(
      body: Row(
        children: [
          // Left panel — conversation list (fixed 280 px)
          SizedBox(
            width: 280,
            child: Column(
              children: [
                AppBar(
                  title: const Text('\ucc44\ud305'),
                  actions: [
                    IconButton(
                      icon: const Icon(Icons.search),
                      onPressed: () => context.push('/search'),
                      tooltip: '\uac80\uc0c9',
                    ),
                    IconButton(
                      icon: const Icon(Icons.edit_square),
                      onPressed: () {},
                      tooltip: '\uc0c8 \ub300\ud654',
                    ),
                  ],
                ),
                Expanded(
                  child: _buildConversationList(
                    conversations,
                    loadState,
                    loadError,
                  ),
                ),
              ],
            ),
          ),
          const VerticalDivider(width: 1),
          // Right panel — chat area
          Expanded(
            child: _selectedConversationId != null
                ? ChatScreen(conversationId: _selectedConversationId!)
                : Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(
                          Icons.chat_bubble_outline,
                          size: 64,
                          color: Theme.of(
                            context,
                          ).colorScheme.onSurfaceVariant.withOpacity(0.5),
                        ),
                        const SizedBox(height: 16),
                        Text(
                          '\ub300\ud654\ub97c \uc120\ud0dd\ud558\uc138\uc694',
                          style: Theme.of(context).textTheme.titleMedium
                              ?.copyWith(
                                color: Theme.of(
                                  context,
                                ).colorScheme.onSurfaceVariant,
                              ),
                        ),
                      ],
                    ),
                  ),
          ),
        ],
      ),
    );
  }

  // ── Shared conversation list ───────────────────────────────────────

  Widget _buildConversationList(
    List conversations,
    ResourceLoadState loadState,
    String? loadError,
  ) {
    if (loadState == ResourceLoadState.loading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (loadState == ResourceLoadState.error) {
      return Center(child: Text(loadError ?? '대화 목록을 불러오지 못했습니다.'));
    }

    if (conversations.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.chat_bubble_outline,
              size: 64,
              color: Theme.of(
                context,
              ).colorScheme.onSurfaceVariant.withOpacity(0.5),
            ),
            const SizedBox(height: 16),
            Text(
              '\uc544\uc9c1 \ub300\ud654\uac00 \uc5c6\uc2b5\ub2c8\ub2e4',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: conversations.length,
      itemBuilder: (context, index) {
        final conversation = conversations[index];
        return ConversationTile(
          conversation: conversation,
          onTap: () {
            setState(() {
              _selectedConversationId = conversation.id;
            });
          },
        );
      },
    );
  }
}
