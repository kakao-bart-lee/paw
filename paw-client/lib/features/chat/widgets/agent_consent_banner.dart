import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

class AgentConsentBanner extends StatelessWidget {
  final List<String> agentNames;

  const AgentConsentBanner({
    super.key,
    required this.agentNames,
  });

  @override
  Widget build(BuildContext context) {
    final names = agentNames.join(', ');
    
    return Container(
      height: 36,
      width: double.infinity,
      color: const Color(0xFF1E2A3A), // Dark blue-gray from decisions.md
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Text('🤖', style: TextStyle(fontSize: 14)),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              '$names이(가) 이 대화를 읽고 있습니다',
              style: const TextStyle(
                color: Color(0xFFFFB300), // Amber text
                fontSize: 12,
                fontWeight: FontWeight.w500,
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          TextButton(
            onPressed: () {
              context.push('/settings');
            },
            style: TextButton.styleFrom(
              padding: EdgeInsets.zero,
              minimumSize: const Size(0, 0),
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            child: const Text(
              '에이전트 관리',
              style: TextStyle(
                color: Color(0xFFFFB300),
                fontSize: 12,
                fontWeight: FontWeight.bold,
                decoration: TextDecoration.underline,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
