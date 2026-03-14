import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/platform/desktop_service.dart';
import '../../../core/theme/app_theme.dart';

class HelpCenterScreen extends StatelessWidget {
  const HelpCenterScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final isDesktopClient = DesktopService().isDesktop;

    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(title: const Text('도움말 센터')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: AppTheme.surface2,
              borderRadius: BorderRadius.circular(AppTheme.radiusXl),
              border: Border.all(
                color: AppTheme.accent.withValues(alpha: 0.24),
              ),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Web/Desktop 품질 체크',
                  style: Theme.of(
                    context,
                  ).textTheme.labelMedium?.copyWith(color: AppTheme.accent),
                ),
                const SizedBox(height: 8),
                Text(
                  '설정, 프로필, 검색, 채팅 상세, Agent 화면을 같은 톤과 구조로 점검하세요.',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 12),
                Text(
                  isDesktopClient
                      ? '데스크톱에서는 2단 레이아웃과 세션 안내 문구까지 확인합니다.'
                      : '웹에서는 브라우저 보안·E2EE 제한 안내가 자연스럽게 보이는지 확인합니다.',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
          const SizedBox(height: 18),
          const _HelpSection(
            title: '추천 점검 순서',
            children: [
              _HelpRow(
                icon: Icons.settings_outlined,
                title: '설정과 프로필',
                subtitle: '상태 카드, 보안 설명, 로그아웃/편집 CTA가 자연스럽게 보이는지 확인',
              ),
              _HelpRow(
                icon: Icons.search_rounded,
                title: '검색과 상세 진입',
                subtitle: '빈 상태, 검색 실패, 대화 상세 진입 흐름이 명확한지 확인',
              ),
              _HelpRow(
                icon: Icons.auto_awesome_outlined,
                title: 'Agent/지원 안내',
                subtitle: '권한 분리 문구와 도움말 링크가 제품 톤을 유지하는지 점검',
                last: true,
              ),
            ],
          ),
          const _HelpSection(
            title: '지원 채널',
            children: [
              _HelpRow(
                icon: Icons.markunread_mailbox_outlined,
                title: '제품 피드백',
                subtitle: 'visual QA 결과와 재현 단계를 함께 남겨 팀 리뷰로 전달',
              ),
              _HelpRow(
                icon: Icons.lock_outline_rounded,
                title: '보안/개인정보 문의',
                subtitle: '세션, 기기, Agent 권한 관련 질문을 우선 분류해서 대응',
              ),
              _HelpRow(
                icon: Icons.desktop_windows_outlined,
                title: '플랫폼 메모',
                subtitle: 'Web/Desktop 전용 클라이언트로 운영 중이며 모바일은 네이티브 앱 사용',
                last: true,
              ),
            ],
          ),
          const SizedBox(height: 18),
          OutlinedButton.icon(
            onPressed: () => context.go('/settings'),
            icon: const Icon(Icons.arrow_back_rounded),
            label: const Text('설정으로 돌아가기'),
          ),
        ],
      ),
    );
  }
}

class _HelpSection extends StatelessWidget {
  const _HelpSection({required this.title, required this.children});

  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 18),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.only(left: 4, bottom: 10),
            child: Text(
              title,
              style: Theme.of(context).textTheme.labelMedium?.copyWith(
                color: AppTheme.mutedText,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
          Container(
            decoration: BoxDecoration(
              color: AppTheme.surface2,
              borderRadius: BorderRadius.circular(AppTheme.radiusMd),
              border: Border.all(color: AppTheme.outline),
            ),
            child: Column(children: children),
          ),
        ],
      ),
    );
  }
}

class _HelpRow extends StatelessWidget {
  const _HelpRow({
    required this.icon,
    required this.title,
    required this.subtitle,
    this.last = false,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final bool last;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 15),
      decoration: BoxDecoration(
        border: last
            ? null
            : const Border(bottom: BorderSide(color: AppTheme.outline)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 38,
            height: 38,
            decoration: BoxDecoration(
              color: AppTheme.surface4,
              borderRadius: BorderRadius.circular(AppTheme.radiusSm),
            ),
            child: Icon(icon, color: AppTheme.accent, size: 20),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(title, style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 4),
                Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
