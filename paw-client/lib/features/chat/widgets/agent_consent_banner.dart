import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';
import '../services/agent_consent_service.dart';

class AgentConsentBanner extends StatefulWidget {
  const AgentConsentBanner({
    super.key,
    required this.agentNames,
    this.conversationId,
  });

  final List<String> agentNames;
  final String? conversationId;

  @override
  State<AgentConsentBanner> createState() => _AgentConsentBannerState();
}

class _AgentConsentBannerState extends State<AgentConsentBanner> {
  final AgentConsentService _consentService = AgentConsentService();

  bool _isLoading = true;
  bool _needsConsent = false;
  bool _isDeclined = false;

  @override
  void initState() {
    super.initState();
    _checkConsentState();
  }

  @override
  void didUpdateWidget(covariant AgentConsentBanner oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.agentNames != widget.agentNames ||
        oldWidget.conversationId != widget.conversationId) {
      _checkConsentState();
    }
  }

  Future<void> _checkConsentState() async {
    setState(() {
      _isLoading = true;
      _needsConsent = false;
      _isDeclined = false;
    });

    final convId = widget.conversationId ?? 'default_conv';
    bool anyNeedsConsent = false;
    bool allDeclined = true;

    for (final agent in widget.agentNames) {
      final consent = await _consentService.getConsent(convId, agent);
      if (consent == null) {
        anyNeedsConsent = true;
        allDeclined = false;
      } else if (consent == true) {
        allDeclined = false;
      }
    }

    if (!mounted) return;
    setState(() {
      _needsConsent = anyNeedsConsent;
      _isDeclined =
          !anyNeedsConsent && allDeclined && widget.agentNames.isNotEmpty;
      _isLoading = false;
    });
  }

  Future<void> _handleConsent(bool allow) async {
    final convId = widget.conversationId ?? 'default_conv';
    for (final agent in widget.agentNames) {
      final consent = await _consentService.getConsent(convId, agent);
      if (consent == null) {
        await _consentService.setConsent(convId, agent, allow);
      }
    }

    if (!mounted) return;
    setState(() {
      _needsConsent = false;
      if (!allow) {
        _isDeclined = true;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Padding(
        padding: EdgeInsets.fromLTRB(16, 10, 16, 0),
        child: SizedBox(height: 52),
      );
    }
    if (_isDeclined) return const SizedBox.shrink();

    final names = widget.agentNames.join(', ');

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 10, 16, 0),
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: AppTheme.agentBubbleDark,
          borderRadius: BorderRadius.circular(18),
          border: Border.all(color: AppTheme.warning.withValues(alpha: 0.24)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  width: 34,
                  height: 34,
                  decoration: BoxDecoration(
                    color: AppTheme.primarySoft,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: const Icon(
                    Icons.auto_awesome_rounded,
                    color: AppTheme.primary,
                    size: 18,
                  ),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    _needsConsent
                        ? '$names이(가) 이 대화에 참여하도록 허용하시겠습니까?'
                        : '$names이(가) 이 대화를 읽고 있습니다',
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              _needsConsent
                  ? '허용 전에는 에이전트가 메시지를 읽거나 응답을 제안하지 않습니다.'
                  : '권한과 접근 범위는 설정에서 언제든 조정할 수 있습니다.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                if (_needsConsent) ...[
                  TextButton(
                    onPressed: () => _handleConsent(false),
                    child: const Text('거부'),
                  ),
                  const SizedBox(width: 8),
                  ElevatedButton(
                    onPressed: () => _handleConsent(true),
                    child: const Text('허용'),
                  ),
                ] else
                  TextButton(
                    onPressed: () => context.push('/settings'),
                    child: const Text('에이전트 관리'),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
