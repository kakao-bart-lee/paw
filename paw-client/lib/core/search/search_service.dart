import 'package:drift/drift.dart';
import '../db/app_database.dart';

/// A single full-text search result.
class SearchResult {
  final String messageId;
  final String conversationId;
  final String snippet;
  final String highlightedContent;
  final DateTime createdAt;

  const SearchResult({
    required this.messageId,
    required this.conversationId,
    required this.snippet,
    required this.highlightedContent,
    required this.createdAt,
  });

  /// Construct from a raw Drift query row.
  factory SearchResult.fromRow(QueryRow row) {
    return SearchResult(
      messageId: row.read<String>('id'),
      conversationId: row.read<String>('conversation_id'),
      snippet: row.read<String>('snippet'),
      highlightedContent: row.read<String>('highlighted_content'),
      createdAt: DateTime.fromMillisecondsSinceEpoch(
        row.read<int>('created_at') * 1000,
      ),
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SearchResult &&
          runtimeType == other.runtimeType &&
          messageId == other.messageId;

  @override
  int get hashCode => messageId.hashCode;

  @override
  String toString() =>
      'SearchResult(messageId: $messageId, conversationId: $conversationId)';
}

/// Full-text search service backed by SQLite FTS5.
class SearchService {
  final AppDatabase _db;

  SearchService(this._db);

  /// Sanitize user input for FTS5 MATCH syntax.
  ///
  /// Escapes special FTS5 characters and wraps each token in double quotes
  /// so arbitrary user input never causes a syntax error.
  static String buildFts5Query(String raw) {
    final trimmed = raw.trim();
    if (trimmed.isEmpty) return '';

    // Split on whitespace, quote each token, join with space (implicit AND).
    final tokens = trimmed.split(RegExp(r'\s+'));
    return tokens
        .where((t) => t.isNotEmpty)
        .map((t) => '"${t.replaceAll('"', '""')}"')
        .join(' ');
  }

  /// Search messages by full-text query.
  ///
  /// Returns up to [limit] results ranked by FTS5 relevance.
  /// Empty or blank [query] returns an empty list immediately.
  Future<List<SearchResult>> search(
    String query, {
    int limit = 50,
  }) async {
    final ftsQuery = buildFts5Query(query);
    if (ftsQuery.isEmpty) return const [];

    final results = await _db.customSelect(
      '''
      SELECT
        m.id,
        m.conversation_id,
        snippet(messages_fts, 0, '[', ']', '...', 32) AS snippet,
        highlight(messages_fts, 0, '[', ']') AS highlighted_content,
        m.created_at
      FROM messages_fts
      JOIN messages_table m ON m.rowid = messages_fts.rowid
      WHERE messages_fts MATCH ?
      ORDER BY rank
      LIMIT ?
      ''',
      variables: [Variable.withString(ftsQuery), Variable.withInt(limit)],
    ).get();

    return results.map(SearchResult.fromRow).toList();
  }
}
