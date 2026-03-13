import 'package:flutter/material.dart';

import '../../../core/theme/app_theme.dart';

enum E2eeBannerType { active, available, agentPresent }

class E2eeBanner extends StatelessWidget {
  const E2eeBanner({
    super.key,
    required this.type,
    this.onActivate,
    this.agentName,
  });

  final E2eeBannerType type;
  final VoidCallback? onActivate;
  final String? agentName;

  @override
  Widget build(BuildContext context) {
    final (icon, title, color, background) = switch (type) {
      E2eeBannerType.active => (
        Icons.lock_rounded,
        '종단간 암호화됨 · Signal Protocol',
        AppTheme.primary,
        AppTheme.primarySoft,
      ),
      E2eeBannerType.available => (
        Icons.lock_open_rounded,
        'E2EE 활성화하시겠습니까?',
        AppTheme.mutedText,
        AppTheme.surface3,
      ),
      E2eeBannerType.agentPresent => (
        Icons.auto_awesome_rounded,
        '${agentName ?? 'Agent'}이(가) 이 대화를 읽고 있습니다',
        AppTheme.warning,
        const Color(0xFF2B2416),
      ),
    };

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 10, 16, 0),
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
        decoration: BoxDecoration(
          color: background,
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: color.withValues(alpha: 0.24)),
        ),
        child: Row(
          children: [
            Icon(icon, color: color, size: 16),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                title,
                style: Theme.of(
                  context,
                ).textTheme.labelMedium?.copyWith(color: color),
              ),
            ),
            if (type == E2eeBannerType.available && onActivate != null)
              TextButton(
                onPressed: onActivate,
                style: TextButton.styleFrom(
                  foregroundColor: AppTheme.strongText,
                  backgroundColor: AppTheme.surface4,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 8,
                  ),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(12),
                  ),
                ),
                child: const Text('활성화'),
              ),
          ],
        ),
      ),
    );
  }
}
