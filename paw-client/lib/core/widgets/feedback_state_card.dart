import 'package:flutter/material.dart';

import '../theme/app_theme.dart';

class FeedbackStateCard extends StatelessWidget {
  const FeedbackStateCard({
    super.key,
    required this.icon,
    required this.title,
    required this.subtitle,
    this.action,
    this.loading = false,
    this.iconSize = 64,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final Widget? action;
  final bool loading;
  final double iconSize;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 420),
          child: Container(
            width: double.infinity,
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: AppTheme.surface2,
              borderRadius: BorderRadius.circular(AppTheme.radiusXl),
              border: Border.all(color: AppTheme.outline),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: iconSize,
                  height: iconSize,
                  decoration: BoxDecoration(
                    color: AppTheme.surface3,
                    borderRadius: BorderRadius.circular(AppTheme.radiusXl),
                    border: Border.all(color: AppTheme.outline),
                  ),
                  child: loading
                      ? const Padding(
                          padding: EdgeInsets.all(18),
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : Icon(icon, size: iconSize * 0.42, color: AppTheme.mutedText),
                ),
                const SizedBox(height: 16),
                Text(title, style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 8),
                Text(
                  subtitle,
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
                if (action != null) ...[const SizedBox(height: 16), action!],
              ],
            ),
          ),
        ),
      ),
    );
  }
}
