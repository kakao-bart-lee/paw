import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../theme/app_theme.dart';

const double kWideShellBreakpoint = 960;

class MainShell extends StatelessWidget {
  final Widget child;
  const MainShell({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    final location = GoRouterState.of(context).uri.path;
    final isWideLayout = MediaQuery.sizeOf(context).width >= kWideShellBreakpoint;
    final showCompactNav = !isWideLayout && !_isDetailRoute(location);

    return Scaffold(
      backgroundColor: AppTheme.background,
      body: Row(
        children: [
          if (isWideLayout)
            _DesktopSidebar(selectedIndex: _selectedIndex(location)),
          Expanded(child: child),
        ],
      ),
      bottomNavigationBar: showCompactNav
          ? _CompactNav(
              selectedIndex: _selectedIndex(location),
              onSelect: (index) => _onDestinationSelected(context, index),
            )
          : null,
    );
  }

  static bool _isDetailRoute(String location) {
    return location.startsWith('/chat/') && !location.endsWith('/verify');
  }

  int _selectedIndex(String location) {
    if (location.startsWith('/chat')) return 0;
    if (location.startsWith('/agent')) return 1;
    if (location.startsWith('/settings')) return 2;
    return 0;
  }

  void _onDestinationSelected(BuildContext context, int index) {
    switch (index) {
      case 0:
        context.go('/chat');
        return;
      case 1:
        context.go('/agent');
        return;
      case 2:
        context.go('/settings');
        return;
    }
  }
}

class _CompactNav extends StatelessWidget {
  const _CompactNav({required this.selectedIndex, required this.onSelect});

  final int selectedIndex;
  final ValueChanged<int> onSelect;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      top: false,
      child: Container(
        margin: const EdgeInsets.fromLTRB(12, 0, 12, 12),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        decoration: BoxDecoration(
          color: AppTheme.surface2,
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: AppTheme.outline),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.18),
              blurRadius: 24,
              offset: const Offset(0, 12),
            ),
          ],
        ),
        child: Row(
          children: [
            _NavItem(
              label: '채팅',
              icon: Icons.chat_bubble_outline_rounded,
              selectedIcon: Icons.chat_bubble_rounded,
              selected: selectedIndex == 0,
              onTap: () => onSelect(0),
            ),
            _NavItem(
              label: 'Agent',
              icon: Icons.auto_awesome_outlined,
              selectedIcon: Icons.auto_awesome_rounded,
              selected: selectedIndex == 1,
              onTap: () => onSelect(1),
            ),
            _NavItem(
              label: '설정',
              icon: Icons.settings_outlined,
              selectedIcon: Icons.settings_rounded,
              selected: selectedIndex == 2,
              onTap: () => onSelect(2),
            ),
          ],
        ),
      ),
    );
  }
}

class _DesktopSidebar extends StatelessWidget {
  const _DesktopSidebar({required this.selectedIndex});

  final int selectedIndex;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 92,
      margin: const EdgeInsets.fromLTRB(16, 16, 0, 16),
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 14),
      decoration: BoxDecoration(
        color: AppTheme.surface2,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: AppTheme.outline),
      ),
      child: Column(
        children: [
          Container(
            width: 48,
            height: 48,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(8),
              gradient: const LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [AppTheme.primarySoft, AppTheme.surface4],
              ),
              border: Border.all(
                color: AppTheme.accent.withValues(alpha: 0.28),
              ),
            ),
            child: const Center(
              child: Text(
                'Pw',
                style: TextStyle(
                  color: AppTheme.accent,
                  fontWeight: FontWeight.w800,
                  letterSpacing: -0.5,
                ),
              ),
            ),
          ),
          const SizedBox(height: 28),
          _SidebarItem(
            label: '채팅',
            icon: Icons.chat_bubble_outline_rounded,
            selectedIcon: Icons.chat_bubble_rounded,
            selected: selectedIndex == 0,
            onTap: () => context.go('/chat'),
          ),
          _SidebarItem(
            label: 'Agent',
            icon: Icons.auto_awesome_outlined,
            selectedIcon: Icons.auto_awesome_rounded,
            selected: selectedIndex == 1,
            onTap: () => context.go('/agent'),
          ),
          _SidebarItem(
            label: '설정',
            icon: Icons.settings_outlined,
            selectedIcon: Icons.settings_rounded,
            selected: selectedIndex == 2,
            onTap: () => context.go('/settings'),
          ),
          const Spacer(),
          InkWell(
            borderRadius: BorderRadius.circular(8),
            onTap: () => context.go('/profile/me'),
            child: Container(
              width: 48,
              height: 48,
              decoration: BoxDecoration(
                color: AppTheme.surface4,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: AppTheme.outline),
              ),
              child: const Center(
                child: Text(
                  'ME',
                  style: TextStyle(
                    color: AppTheme.strongText,
                    fontWeight: FontWeight.w700,
                    fontSize: 12,
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _NavItem extends StatelessWidget {
  const _NavItem({
    required this.label,
    required this.icon,
    required this.selectedIcon,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final IconData icon;
  final IconData selectedIcon;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Semantics(
        button: true,
        selected: selected,
        child: InkWell(
          borderRadius: BorderRadius.circular(8),
          onTap: onTap,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 180),
            padding: const EdgeInsets.symmetric(vertical: 10),
            decoration: BoxDecoration(
              color: selected ? AppTheme.primarySoft : Colors.transparent,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  selected ? selectedIcon : icon,
                  color: selected ? AppTheme.accent : AppTheme.mutedText,
                ),
                const SizedBox(height: 4),
                Text(
                  label,
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: selected ? AppTheme.accent : AppTheme.mutedText,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _SidebarItem extends StatelessWidget {
  const _SidebarItem({
    required this.label,
    required this.icon,
    required this.selectedIcon,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final IconData icon;
  final IconData selectedIcon;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        borderRadius: BorderRadius.circular(8),
        onTap: onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 180),
          width: double.infinity,
          padding: const EdgeInsets.symmetric(vertical: 12),
          decoration: BoxDecoration(
            color: selected ? AppTheme.primarySoft : Colors.transparent,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Column(
            children: [
              Icon(
                selected ? selectedIcon : icon,
                color: selected ? AppTheme.accent : AppTheme.mutedText,
              ),
              const SizedBox(height: 4),
              Text(
                label,
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: selected ? AppTheme.accent : AppTheme.mutedText,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
