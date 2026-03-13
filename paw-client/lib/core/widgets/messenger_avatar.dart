import 'package:flutter/material.dart';

import '../theme/app_theme.dart';

class MessengerAvatar extends StatelessWidget {
  const MessengerAvatar({
    super.key,
    required this.name,
    this.imageUrl,
    this.size = 48,
    this.isAgent = false,
    this.isOnline = false,
    this.showPresence = true,
  });

  final String name;
  final String? imageUrl;
  final double size;
  final bool isAgent;
  final bool isOnline;
  final bool showPresence;

  @override
  Widget build(BuildContext context) {
    final radius = BorderRadius.circular(size * 0.32);
    final initials = _initials(name);

    return SizedBox(
      width: size,
      height: size,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Positioned.fill(
            child: DecoratedBox(
              decoration: BoxDecoration(
                borderRadius: radius,
                gradient: LinearGradient(
                  begin: Alignment.topLeft,
                  end: Alignment.bottomRight,
                  colors: isAgent
                      ? const [AppTheme.primarySoft, AppTheme.surface4]
                      : const [AppTheme.surface4, AppTheme.surface3],
                ),
                border: Border.all(
                  color: isAgent
                      ? AppTheme.primary.withValues(alpha: 0.28)
                      : AppTheme.outline,
                ),
              ),
              child: ClipRRect(
                borderRadius: radius,
                child: imageUrl != null && imageUrl!.isNotEmpty
                    ? Image.network(
                        imageUrl!,
                        fit: BoxFit.cover,
                        errorBuilder: (_, __, ___) => _AvatarFallback(
                          initials: initials,
                          isAgent: isAgent,
                          fontSize: size * 0.3,
                        ),
                      )
                    : _AvatarFallback(
                        initials: initials,
                        isAgent: isAgent,
                        fontSize: size * 0.3,
                      ),
              ),
            ),
          ),
          if (isAgent)
            Positioned(
              top: -2,
              right: -2,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
                decoration: BoxDecoration(
                  color: AppTheme.primary,
                  borderRadius: BorderRadius.circular(999),
                  border: Border.all(color: AppTheme.background, width: 1.5),
                ),
                child: Text(
                  'AI',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: AppTheme.background,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ),
            ),
          if (showPresence && isOnline)
            Positioned(
              right: -1,
              bottom: -1,
              child: Container(
                width: size * 0.22,
                height: size * 0.22,
                decoration: BoxDecoration(
                  color: AppTheme.online,
                  shape: BoxShape.circle,
                  border: Border.all(color: AppTheme.surface2, width: 2),
                ),
              ),
            ),
        ],
      ),
    );
  }

  static String _initials(String value) {
    final cleaned = value.trim();
    if (cleaned.isEmpty) return '?';

    final parts = cleaned
        .split(RegExp(r'\s+'))
        .where((part) => part.isNotEmpty)
        .toList();
    if (parts.length == 1) {
      final word = parts.first;
      return word.substring(0, word.length >= 2 ? 2 : 1).toUpperCase();
    }
    return (parts.first.substring(0, 1) + parts.last.substring(0, 1))
        .toUpperCase();
  }
}

class _AvatarFallback extends StatelessWidget {
  const _AvatarFallback({
    required this.initials,
    required this.isAgent,
    required this.fontSize,
  });

  final String initials;
  final bool isAgent;
  final double fontSize;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Text(
        initials,
        style: Theme.of(context).textTheme.titleMedium?.copyWith(
          fontSize: fontSize,
          fontWeight: FontWeight.w800,
          color: isAgent ? AppTheme.primary : AppTheme.strongText,
          letterSpacing: -0.4,
        ),
      ),
    );
  }
}
