import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/platform/desktop_service.dart';
import '../../../core/theme/app_theme.dart';
import '../../auth/providers/auth_provider.dart';

class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  bool notifications = true;
  bool previews = true;
  bool readReceipts = true;
  bool typingIndicators = true;
  bool darkMode = true;

  @override
  Widget build(BuildContext context) {
    final isDesktopClient = DesktopService().isDesktop;
    final sessionSecurityTitle = isDesktopClient ? '데스크톱 세션' : '웹 세션';
    final sessionSecuritySubtitle = isDesktopClient
        ? '앱 잠금은 네이티브 모바일 앱에서 제공됩니다'
        : '브라우저와 운영체제의 보안 설정을 따릅니다';

    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('설정', style: Theme.of(context).textTheme.titleLarge),
            Text(
              '개인정보, 알림, 에이전트 권한을 관리하세요',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 110),
        children: [
          InkWell(
            borderRadius: BorderRadius.circular(24),
            onTap: () => context.go('/profile/me'),
            child: Ink(
              padding: const EdgeInsets.all(18),
              decoration: BoxDecoration(
                color: AppTheme.surface2,
                borderRadius: BorderRadius.circular(24),
                border: Border.all(color: AppTheme.outline),
              ),
              child: Row(
                children: [
                  Container(
                    width: 54,
                    height: 54,
                    decoration: BoxDecoration(
                      color: AppTheme.primarySoft,
                      borderRadius: BorderRadius.circular(18),
                      border: Border.all(
                        color: AppTheme.primary.withValues(alpha: 0.28),
                      ),
                    ),
                    child: const Center(
                      child: Text(
                        'ME',
                        style: TextStyle(
                          color: AppTheme.primary,
                          fontWeight: FontWeight.w800,
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 14),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          '내 프로필',
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                        const SizedBox(height: 4),
                        Text(
                          '이름, 전화번호, 프로필 이미지를 편집합니다',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ],
                    ),
                  ),
                  const Icon(Icons.chevron_right_rounded),
                ],
              ),
            ),
          ),
          const SizedBox(height: 20),
          _SettingsSection(
            title: '알림',
            children: [
              _SettingsToggleRow(
                icon: Icons.notifications_outlined,
                title: '알림 받기',
                subtitle: '메시지, 멘션, 중요 활동 알림',
                value: notifications,
                onChanged: (value) => setState(() => notifications = value),
              ),
              _SettingsToggleRow(
                icon: Icons.mark_chat_unread_outlined,
                title: '잠금 화면 미리보기',
                subtitle: '잠금 화면에 메시지 내용을 표시합니다',
                value: previews,
                onChanged: (value) => setState(() => previews = value),
                last: true,
              ),
            ],
          ),
          _SettingsSection(
            title: '개인정보',
            children: [
              _SettingsToggleRow(
                icon: Icons.done_all_rounded,
                title: '읽음 확인',
                subtitle: '상대방에게 읽음 상태를 표시합니다',
                value: readReceipts,
                onChanged: (value) => setState(() => readReceipts = value),
              ),
              _SettingsToggleRow(
                icon: Icons.keyboard_alt_outlined,
                title: '입력 중 표시',
                subtitle: '대화 중 입력 상태를 공유합니다',
                value: typingIndicators,
                onChanged: (value) => setState(() => typingIndicators = value),
              ),
              _SettingsActionRow(
                icon: Icons.language_rounded,
                title: '언어',
                subtitle: '한국어',
                onTap: () {},
                last: true,
              ),
            ],
          ),
          _SettingsSection(
            title: '보안',
            children: [
              _SettingsActionRow(
                icon: Icons.verified_user_outlined,
                title: '보안 점검',
                subtitle: '연결된 기기와 최근 세션 확인',
                onTap: () {},
              ),
              _SettingsActionRow(
                icon: isDesktopClient
                    ? Icons.desktop_windows_outlined
                    : Icons.web_asset_rounded,
                title: sessionSecurityTitle,
                subtitle: sessionSecuritySubtitle,
                onTap: () {},
              ),
              _SettingsToggleRow(
                icon: Icons.dark_mode_outlined,
                title: '다크 모드',
                subtitle: 'Aether 스타일의 어두운 테마 유지',
                value: darkMode,
                onChanged: (value) => setState(() => darkMode = value),
                last: true,
              ),
            ],
          ),
          if (!kIsWeb && isDesktopClient)
            const _SettingsSection(
              title: '데스크톱',
              children: [
                _SettingsInfoRow(
                  icon: Icons.keyboard_command_key_rounded,
                  title: '키보드 단축키',
                  subtitle: 'macOS 데스크톱 gate에서 단축키 등록 상태를 점검합니다',
                  last: true,
                ),
              ],
            ),
          _SettingsSection(
            title: 'AI & Agent',
            children: [
              _SettingsActionRow(
                icon: Icons.auto_awesome_outlined,
                title: 'Agent 권한',
                subtitle: '대화별 접근 범위와 동의 기록 관리',
                onTap: () => context.go('/agent'),
              ),
              _SettingsActionRow(
                icon: Icons.shield_outlined,
                title: '데이터 사용 정책',
                subtitle: 'AI 개선에 활용되는 항목을 제어합니다',
                onTap: () {},
                last: true,
              ),
            ],
          ),
          const SizedBox(height: 8),
          OutlinedButton.icon(
            onPressed: () async {
              await ref.read(authNotifierProvider.notifier).logout();
              if (!context.mounted) return;
              context.go('/login');
            },
            style: OutlinedButton.styleFrom(
              foregroundColor: AppTheme.danger,
              side: const BorderSide(color: Color(0x55FF7A7A)),
            ),
            icon: const Icon(Icons.logout_rounded),
            label: const Text('로그아웃'),
          ),
        ],
      ),
    );
  }
}

class _SettingsInfoRow extends StatelessWidget {
  const _SettingsInfoRow({
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
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
          child: Row(
            children: [
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  color: AppTheme.surface3,
                  borderRadius: BorderRadius.circular(14),
                  border: Border.all(color: AppTheme.outline),
                ),
                child: Icon(icon, color: AppTheme.mutedText),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(title, style: Theme.of(context).textTheme.titleSmall),
                    const SizedBox(height: 4),
                    Text(
                      subtitle,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        if (!last)
          const Divider(height: 1, indent: 16, endIndent: 16),
      ],
    );
  }
}

class _SettingsSection extends StatelessWidget {
  const _SettingsSection({required this.title, required this.children});

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
              borderRadius: BorderRadius.circular(24),
              border: Border.all(color: AppTheme.outline),
            ),
            child: Column(children: children),
          ),
        ],
      ),
    );
  }
}

class _SettingsActionRow extends StatelessWidget {
  const _SettingsActionRow({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.onTap,
    this.last = false,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final VoidCallback onTap;
  final bool last;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 15),
        decoration: BoxDecoration(
          border: last
              ? null
              : const Border(bottom: BorderSide(color: AppTheme.outline)),
        ),
        child: Row(
          children: [
            _SettingsIcon(icon: icon),
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
            const Icon(Icons.chevron_right_rounded),
          ],
        ),
      ),
    );
  }
}

class _SettingsToggleRow extends StatelessWidget {
  const _SettingsToggleRow({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.value,
    required this.onChanged,
    this.last = false,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final bool value;
  final ValueChanged<bool> onChanged;
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
        children: [
          _SettingsIcon(icon: icon),
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
          Switch.adaptive(value: value, onChanged: onChanged),
        ],
      ),
    );
  }
}

class _SettingsIcon extends StatelessWidget {
  const _SettingsIcon({required this.icon});

  final IconData icon;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 38,
      height: 38,
      decoration: BoxDecoration(
        color: AppTheme.surface4,
        borderRadius: BorderRadius.circular(14),
      ),
      child: Icon(icon, size: 20, color: AppTheme.primary),
    );
  }
}
