import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../../../core/di/service_locator.dart';
import '../../../core/search/search_service.dart';

/// Full-text message search screen.
class SearchScreen extends StatefulWidget {
  const SearchScreen({super.key});

  @override
  State<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends State<SearchScreen> {
  final _controller = TextEditingController();
  final _focusNode = FocusNode();
  Timer? _debounce;

  List<SearchResult> _results = const [];
  bool _loading = false;
  String _lastQuery = '';

  @override
  void initState() {
    super.initState();
    // Auto-focus the search field on open.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _focusNode.requestFocus();
    });
  }

  @override
  void dispose() {
    _debounce?.cancel();
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _onQueryChanged(String query) {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 300), () {
      _performSearch(query);
    });
  }

  Future<void> _performSearch(String query) async {
    final trimmed = query.trim();
    if (trimmed == _lastQuery) return;
    _lastQuery = trimmed;

    if (trimmed.isEmpty) {
      setState(() {
        _results = const [];
        _loading = false;
      });
      return;
    }

    setState(() => _loading = true);

    try {
      final searchService = getIt<SearchService>();
      final results = await searchService.search(trimmed);
      // Only apply if still the latest query.
      if (trimmed == _lastQuery) {
        setState(() {
          _results = results;
          _loading = false;
        });
      }
    } catch (_) {
      if (trimmed == _lastQuery) {
        setState(() => _loading = false);
      }
    }
  }

  void _navigateToResult(SearchResult result) {
    context.push('/chat/${result.conversationId}');
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Scaffold(
      appBar: AppBar(
        titleSpacing: 0,
        title: TextField(
          controller: _controller,
          focusNode: _focusNode,
          onChanged: _onQueryChanged,
          style: theme.textTheme.bodyLarge,
          decoration: InputDecoration(
            hintText: '메시지 검색...',
            border: InputBorder.none,
            enabledBorder: InputBorder.none,
            focusedBorder: InputBorder.none,
            filled: false,
            hintStyle: TextStyle(color: colorScheme.onSurfaceVariant),
            contentPadding: const EdgeInsets.symmetric(vertical: 12),
          ),
        ),
        actions: [
          if (_controller.text.isNotEmpty)
            IconButton(
              icon: const Icon(Icons.close),
              onPressed: () {
                _controller.clear();
                _onQueryChanged('');
              },
              tooltip: '지우기',
            ),
        ],
      ),
      body: _buildBody(theme, colorScheme),
    );
  }

  Widget _buildBody(ThemeData theme, ColorScheme colorScheme) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_lastQuery.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.search,
              size: 64,
              color: colorScheme.onSurfaceVariant.withOpacity(0.4),
            ),
            const SizedBox(height: 16),
            Text(
              '대화 내용을 검색하세요',
              style: theme.textTheme.titleMedium?.copyWith(
                color: colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      );
    }

    if (_results.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.search_off,
              size: 64,
              color: colorScheme.onSurfaceVariant.withOpacity(0.4),
            ),
            const SizedBox(height: 16),
            Text(
              '검색 결과가 없습니다',
              style: theme.textTheme.titleMedium?.copyWith(
                color: colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: _results.length,
      separatorBuilder: (_, __) => const Divider(height: 1, indent: 16, endIndent: 16),
      itemBuilder: (context, index) {
        final result = _results[index];
        return _SearchResultTile(
          result: result,
          onTap: () => _navigateToResult(result),
        );
      },
    );
  }
}

class _SearchResultTile extends StatelessWidget {
  final SearchResult result;
  final VoidCallback onTap;

  const _SearchResultTile({required this.result, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return ListTile(
      leading: CircleAvatar(
        backgroundColor: colorScheme.primary.withOpacity(0.15),
        child: Icon(Icons.message, color: colorScheme.primary, size: 20),
      ),
      title: _buildHighlightedText(
        result.highlightedContent,
        theme,
        colorScheme,
      ),
      subtitle: Text(
        _formatDate(result.createdAt),
        style: theme.textTheme.bodySmall,
      ),
      onTap: onTap,
    );
  }

  /// Parse `[highlight]` markers from FTS5 highlight() into styled spans.
  Widget _buildHighlightedText(
    String text,
    ThemeData theme,
    ColorScheme colorScheme,
  ) {
    final spans = <InlineSpan>[];
    final regex = RegExp(r'\[(.+?)\]');
    int cursor = 0;

    for (final match in regex.allMatches(text)) {
      if (match.start > cursor) {
        spans.add(TextSpan(text: text.substring(cursor, match.start)));
      }
      spans.add(TextSpan(
        text: match.group(1),
        style: TextStyle(
          color: colorScheme.primary,
          fontWeight: FontWeight.bold,
        ),
      ));
      cursor = match.end;
    }
    if (cursor < text.length) {
      spans.add(TextSpan(text: text.substring(cursor)));
    }

    return RichText(
      maxLines: 2,
      overflow: TextOverflow.ellipsis,
      text: TextSpan(
        style: theme.textTheme.bodyMedium,
        children: spans,
      ),
    );
  }

  String _formatDate(DateTime dt) {
    final now = DateTime.now();
    final diff = now.difference(dt);
    if (diff.inDays == 0) {
      return '${dt.hour.toString().padLeft(2, '0')}:${dt.minute.toString().padLeft(2, '0')}';
    } else if (diff.inDays < 7) {
      return '${diff.inDays}일 전';
    } else {
      return '${dt.year}.${dt.month.toString().padLeft(2, '0')}.${dt.day.toString().padLeft(2, '0')}';
    }
  }
}
