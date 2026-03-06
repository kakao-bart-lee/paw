import 'package:drift/drift.dart';
import 'package:drift/native.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/core/db/app_database.dart';
import 'package:paw_client/core/db/tables/messages_table.dart';
import 'package:paw_client/core/search/search_service.dart';

void main() {
  // ── buildFts5Query tests ─────────────────────────────────────────────

  group('SearchService.buildFts5Query', () {
    test('returns empty string for blank input', () {
      expect(SearchService.buildFts5Query(''), '');
      expect(SearchService.buildFts5Query('   '), '');
    });

    test('quotes single token', () {
      expect(SearchService.buildFts5Query('hello'), '"hello"');
    });

    test('quotes multiple tokens with implicit AND', () {
      expect(
        SearchService.buildFts5Query('hello world'),
        '"hello" "world"',
      );
    });

    test('escapes double-quote characters in tokens', () {
      expect(
        SearchService.buildFts5Query('say "hi"'),
        '"say" """hi"""',
      );
    });

    test('handles Korean text correctly', () {
      expect(
        SearchService.buildFts5Query('안녕 세계'),
        '"안녕" "세계"',
      );
    });

    test('collapses extra whitespace', () {
      expect(
        SearchService.buildFts5Query('  foo   bar  '),
        '"foo" "bar"',
      );
    });
  });

  // ── SearchResult model tests ─────────────────────────────────────────

  group('SearchResult', () {
    test('equality based on messageId', () {
      final a = SearchResult(
        messageId: 'msg-1',
        conversationId: 'conv-1',
        snippet: 'hello',
        highlightedContent: '[hello]',
        createdAt: DateTime(2025, 1, 1),
      );
      final b = SearchResult(
        messageId: 'msg-1',
        conversationId: 'conv-2',
        snippet: 'different',
        highlightedContent: '[different]',
        createdAt: DateTime(2025, 6, 1),
      );
      expect(a, equals(b));
      expect(a.hashCode, equals(b.hashCode));
    });

    test('inequality for different messageIds', () {
      final a = SearchResult(
        messageId: 'msg-1',
        conversationId: 'conv-1',
        snippet: 'hello',
        highlightedContent: '[hello]',
        createdAt: DateTime(2025, 1, 1),
      );
      final b = SearchResult(
        messageId: 'msg-2',
        conversationId: 'conv-1',
        snippet: 'hello',
        highlightedContent: '[hello]',
        createdAt: DateTime(2025, 1, 1),
      );
      expect(a, isNot(equals(b)));
    });

    test('toString includes messageId and conversationId', () {
      final result = SearchResult(
        messageId: 'msg-1',
        conversationId: 'conv-1',
        snippet: 'test',
        highlightedContent: 'test',
        createdAt: DateTime(2025, 1, 1),
      );
      expect(result.toString(), contains('msg-1'));
      expect(result.toString(), contains('conv-1'));
    });
  });

  // ── Empty query handling (no DB needed) ──────────────────────────────

  group('SearchService.search — empty queries', () {
    late AppDatabase db;
    late SearchService service;

    setUp(() {
      db = AppDatabase.forTesting(NativeDatabase.memory());
      service = SearchService(db);
    });

    tearDown(() => db.close());

    test('empty string returns empty list immediately', () async {
      final results = await service.search('');
      expect(results, isEmpty);
    });

    test('whitespace-only returns empty list', () async {
      final results = await service.search('   ');
      expect(results, isEmpty);
    });
  });

  // ── FTS5 integration test ────────────────────────────────────────────

  group('SearchService.search — FTS5 integration', () {
    late AppDatabase db;
    late SearchService service;

    setUp(() async {
      db = AppDatabase.forTesting(NativeDatabase.memory());
      // Wait for migrations (which create FTS5 table + triggers).
      await db.customSelect('SELECT 1').get();
      service = SearchService(db);
    });

    tearDown(() => db.close());

    Future<void> insertMessage({
      required String id,
      required String conversationId,
      required String content,
    }) async {
      await db.into(db.messagesTable).insert(
            MessagesTableCompanion.insert(
              id: id,
              conversationId: conversationId,
              senderId: 'user-1',
              content: content,
              seq: 1,
              createdAt: DateTime.now(),
            ),
          );
    }

    test('finds messages matching query', () async {
      await insertMessage(
        id: 'msg-1',
        conversationId: 'conv-1',
        content: 'Hello world flutter',
      );
      await insertMessage(
        id: 'msg-2',
        conversationId: 'conv-2',
        content: 'Goodbye world dart',
      );
      await insertMessage(
        id: 'msg-3',
        conversationId: 'conv-1',
        content: 'Unrelated message',
      );

      final results = await service.search('world');
      expect(results.length, 2);
      final ids = results.map((r) => r.messageId).toSet();
      expect(ids, containsAll(['msg-1', 'msg-2']));
    });

    test('respects limit parameter', () async {
      for (var i = 0; i < 10; i++) {
        await insertMessage(
          id: 'msg-$i',
          conversationId: 'conv-1',
          content: 'Searchable content item $i',
        );
      }

      final results = await service.search('searchable', limit: 3);
      expect(results.length, 3);
    });

    test('returns highlighted content with markers', () async {
      await insertMessage(
        id: 'msg-1',
        conversationId: 'conv-1',
        content: 'The quick brown fox',
      );

      final results = await service.search('quick');
      expect(results.length, 1);
      expect(results.first.highlightedContent, contains('[quick]'));
    });
  });
}
