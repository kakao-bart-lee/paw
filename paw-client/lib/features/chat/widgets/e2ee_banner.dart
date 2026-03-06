import 'package:flutter/material.dart';

enum E2eeBannerType { active, available, agentPresent }

class E2eeBanner extends StatelessWidget {
  final E2eeBannerType type;
  final VoidCallback? onActivate;
  final String? agentName;

  const E2eeBanner({
    super.key,
    required this.type,
    this.onActivate,
    this.agentName,
  });

  @override
  Widget build(BuildContext context) {
    Color backgroundColor;
    Color textColor;
    String text;
    IconData icon;

    switch (type) {
      case E2eeBannerType.active:
        backgroundColor = const Color(0xFF1B3A2D);
        textColor = const Color(0xFF4CAF50);
        text = '종단간 암호화됨 · Signal Protocol';
        icon = Icons.lock;
        break;
      case E2eeBannerType.available:
        backgroundColor = const Color(0xFF1E1E1E);
        textColor = Colors.grey;
        text = 'E2EE 활성화하시겠습니까?';
        icon = Icons.lock_open;
        break;
      case E2eeBannerType.agentPresent:
        backgroundColor = const Color(0xFF2D2200);
        textColor = const Color(0xFFFFB300);
        text = '${agentName ?? 'Agent'}이(가) 이 대화를 읽고 있습니다';
        icon = Icons.smart_toy;
        break;
    }

    return Container(
      height: 36,
      width: double.infinity,
      color: backgroundColor,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(icon, color: textColor, size: 14),
          const SizedBox(width: 6),
          Text(
            text,
            style: TextStyle(
              color: textColor,
              fontSize: 12,
              fontWeight: FontWeight.w500,
            ),
          ),
          if (type == E2eeBannerType.available && onActivate != null) ...[
            const SizedBox(width: 8),
            TextButton(
              onPressed: onActivate,
              style: TextButton.styleFrom(
                padding: EdgeInsets.zero,
                minimumSize: const Size(0, 0),
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
              child: Text(
                '활성화',
                style: TextStyle(
                  color: textColor,
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  decoration: TextDecoration.underline,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}
