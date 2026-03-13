import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';

class AgentScreen extends StatefulWidget {
  const AgentScreen({super.key});

  @override
  State<AgentScreen> createState() => _AgentScreenState();
}

class _AgentScreenState extends State<AgentScreen> {
  final _searchController = TextEditingController();
  String _category = '전체';

  static const _categories = ['전체', '생산성', '코드', '리서치', '보안'];
  static const _agents = <_AgentItem>[
    _AgentItem(
      name: 'Aria',
      category: '생산성',
      description: '회의 요약, 일정 정리, 맥락 유지까지 맡는 개인 AI 어시스턴트',
      users: '2.1M',
      rating: '4.9',
      featured: true,
      installed: true,
    ),
    _AgentItem(
      name: 'Code Assistant',
      category: '코드',
      description: '리뷰, 디버깅, 리팩터링 제안에 특화된 개발 파트너',
      users: '890K',
      rating: '4.8',
      installed: true,
    ),
    _AgentItem(
      name: 'Research Agent',
      category: '리서치',
      description: '논문, 뉴스, 내부 문서를 엮어 근거 중심 브리핑을 제공합니다',
      users: '340K',
      rating: '4.7',
    ),
    _AgentItem(
      name: 'Privacy Sentinel',
      category: '보안',
      description: '권한 스코프와 민감정보 노출 가능성을 선제적으로 감시합니다',
      users: '120K',
      rating: '4.6',
    ),
  ];

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final query = _searchController.text.trim().toLowerCase();
    final filtered = _agents.where((agent) {
      final matchesCategory = _category == '전체' || agent.category == _category;
      final matchesSearch =
          query.isEmpty ||
          agent.name.toLowerCase().contains(query) ||
          agent.description.toLowerCase().contains(query);
      return matchesCategory && matchesSearch;
    }).toList();

    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Agent', style: Theme.of(context).textTheme.titleLarge),
            Text(
              '메신저 안에서 바로 쓰는 AI 도구 모음',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 110),
        children: [
          Container(
            padding: const EdgeInsets.all(18),
            decoration: BoxDecoration(
              color: AppTheme.surface2,
              borderRadius: BorderRadius.circular(28),
              border: Border.all(
                color: AppTheme.primary.withValues(alpha: 0.24),
              ),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '추천 Agent',
                  style: Theme.of(
                    context,
                  ).textTheme.labelMedium?.copyWith(color: AppTheme.primary),
                ),
                const SizedBox(height: 8),
                Text(
                  'Aria와 함께 대화 요약, 액션 아이템 정리, 답장 초안을 즉시 생성하세요.',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 10),
                Text(
                  '권한은 대화별로 분리되고, 동의 없이 메시지에 접근하지 않습니다.',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
                const SizedBox(height: 16),
                ElevatedButton.icon(
                  onPressed: () => context.go('/chat'),
                  icon: const Icon(Icons.auto_awesome_rounded),
                  label: const Text('Agent와 대화 열기'),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _searchController,
            onChanged: (_) => setState(() {}),
            decoration: const InputDecoration(
              prefixIcon: Icon(Icons.search_rounded),
              hintText: 'Agent 검색',
            ),
          ),
          const SizedBox(height: 14),
          SizedBox(
            height: 36,
            child: ListView.separated(
              scrollDirection: Axis.horizontal,
              itemCount: _categories.length,
              separatorBuilder: (_, __) => const SizedBox(width: 8),
              itemBuilder: (context, index) {
                final category = _categories[index];
                final selected = category == _category;
                return ChoiceChip(
                  label: Text(category),
                  selected: selected,
                  onSelected: (_) => setState(() => _category = category),
                );
              },
            ),
          ),
          const SizedBox(height: 18),
          ...filtered.map(
            (agent) => Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: _AgentCard(agent: agent),
            ),
          ),
        ],
      ),
    );
  }
}

class _AgentCard extends StatelessWidget {
  const _AgentCard({required this.agent});

  final _AgentItem agent;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: AppTheme.surface2,
        borderRadius: BorderRadius.circular(24),
        border: Border.all(
          color: agent.featured
              ? AppTheme.primary.withValues(alpha: 0.24)
              : AppTheme.outline,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: AppTheme.primarySoft,
                  borderRadius: BorderRadius.circular(16),
                ),
                child: Center(
                  child: Text(
                    agent.name.substring(0, 2).toUpperCase(),
                    style: const TextStyle(
                      color: AppTheme.primary,
                      fontWeight: FontWeight.w800,
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Flexible(
                          child: Text(
                            agent.name,
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                        ),
                        if (agent.featured) ...[
                          const SizedBox(width: 6),
                          const Icon(
                            Icons.auto_awesome_rounded,
                            size: 16,
                            color: AppTheme.primary,
                          ),
                        ],
                      ],
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '${agent.category} · ${agent.users} users · ★ ${agent.rating}',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
              FilledButton.tonal(
                onPressed: () {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text(
                        agent.installed
                            ? '${agent.name} 대화를 여는 기능은 곧 연결됩니다.'
                            : '${agent.name} 추가 기능은 곧 제공됩니다.',
                      ),
                    ),
                  );
                },
                child: Text(agent.installed ? '열기' : '추가'),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Text(agent.description, style: Theme.of(context).textTheme.bodySmall),
        ],
      ),
    );
  }
}

class _AgentItem {
  const _AgentItem({
    required this.name,
    required this.category,
    required this.description,
    required this.users,
    required this.rating,
    this.featured = false,
    this.installed = false,
  });

  final String name;
  final String category;
  final String description;
  final String users;
  final String rating;
  final bool featured;
  final bool installed;
}
