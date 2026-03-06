import 'package:flutter/material.dart';

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
        color: isComplete
            ? Colors.green.withValues(alpha: 0.1)
            : Colors.amber.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(
          color: isComplete
              ? Colors.green.withValues(alpha: 0.3)
              : Colors.amber.withValues(alpha: 0.3),
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
                valueColor: AlwaysStoppedAnimation<Color>(Colors.amber),
              ),
            ),
            const SizedBox(width: 8),
            Text(
              '🔧 $label...',
              style: const TextStyle(
                color: Colors.amber,
                fontSize: 12,
                fontWeight: FontWeight.w500,
              ),
            ),
          ] else ...[
            const Icon(
              Icons.check_circle,
              size: 14,
              color: Colors.green,
            ),
            const SizedBox(width: 6),
            Text(
              '✅ $label',
              style: const TextStyle(
                color: Colors.green,
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
