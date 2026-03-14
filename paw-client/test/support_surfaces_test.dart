import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:paw_client/features/agent/screens/agent_screen.dart';
import 'package:paw_client/features/settings/screens/help_center_screen.dart';

void main() {
  testWidgets('HelpCenterScreen shows QA guidance', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: HelpCenterScreen()));

    expect(find.text('도움말 센터'), findsOneWidget);
    expect(find.textContaining('Web/Desktop 품질 체크'), findsOneWidget);
    expect(find.textContaining('설정, 프로필, 검색, 채팅 상세'), findsOneWidget);
  });

  testWidgets('AgentScreen shows empty-state reset when no search matches', (
    tester,
  ) async {
    final router = GoRouter(
      routes: [
        GoRoute(path: '/', builder: (context, state) => const AgentScreen()),
      ],
    );

    await tester.pumpWidget(MaterialApp.router(routerConfig: router));

    await tester.enterText(find.byType(TextField), 'zzz-no-match');
    await tester.pumpAndSettle();

    expect(find.text('일치하는 Agent가 없습니다'), findsOneWidget);
    expect(find.text('필터 초기화'), findsOneWidget);
  });
}
