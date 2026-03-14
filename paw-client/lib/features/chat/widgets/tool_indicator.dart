import 'package:flutter/material.dart';

import '../../../core/theme/app_theme.dart';

class ToolIndicator extends StatelessWidget {
  final String toolName;
  final String label;
  final bool isComplete;

  const ToolIndicator({
    super.key,
    required this.toolName,
    required this.label,
    required this.isComplete,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 300),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: isComplete ? AppTheme.successSurface : AppTheme.warningSurface,
        borderRadius: BorderRadius.circular(AppTheme.radiusXl),
        border: Border.all(
          color: isComplete
              ? AppTheme.success.withValues(alpha: 0.28)
              : AppTheme.warning.withValues(alpha: 0.28),
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (!isComplete) ...[
            const SizedBox(
              width: 12,
              height: 12,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                valueColor: AlwaysStoppedAnimation<Color>(AppTheme.warning),
              ),
            ),
            const SizedBox(width: 8),
            Text(
              '🔧 $label...',
              style: const TextStyle(
                color: AppTheme.warning,
                fontSize: 12,
                fontWeight: FontWeight.w500,
              ),
            ),
          ] else ...[
            const Icon(Icons.check_circle, size: 14, color: AppTheme.success),
            const SizedBox(width: 6),
            Text(
              '✅ $label',
              style: const TextStyle(
                color: AppTheme.success,
                fontSize: 12,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ],
      ),
    );
  }
}
