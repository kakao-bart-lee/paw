import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../services/agent_consent_service.dart';

class AgentConsentBanner extends StatefulWidget {
  final List<String> agentNames;
  final String? conversationId;

  const AgentConsentBanner({
    super.key,
    required this.agentNames,
    this.conversationId,
  });

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

    if (mounted) {
      setState(() {
        _needsConsent = anyNeedsConsent;
        _isDeclined = !anyNeedsConsent && allDeclined && widget.agentNames.isNotEmpty;
        _isLoading = false;
      });
    }
  }

  Future<void> _handleConsent(bool allow) async {
    final convId = widget.conversationId ?? 'default_conv';
    
    for (final agent in widget.agentNames) {
      final consent = await _consentService.getConsent(convId, agent);
      if (consent == null) {
        await _consentService.setConsent(convId, agent, allow);
      }
    }

    if (mounted) {
      setState(() {
        _needsConsent = false;
        if (!allow) {
          _isDeclined = true;
        }
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const SizedBox(height: 36);
    }

    if (_isDeclined) {
      return const SizedBox.shrink();
    }

    final names = widget.agentNames.join(', ');

    if (_needsConsent) {
      return Container(
        width: double.infinity,
        color: const Color(0xFF1E2A3A),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                const Text('🤖', style: TextStyle(fontSize: 16)),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    '$names이(가) 이 대화에 참여하도록 허용하시겠습니까?',
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 13,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(
                  onPressed: () => _handleConsent(false),
                  style: TextButton.styleFrom(
                    foregroundColor: Colors.white70,
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                  ),
                  child: const Text('거부'),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  onPressed: () => _handleConsent(true),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFFFFB300),
                    foregroundColor: Colors.black87,
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                  ),
                  child: const Text('허용', style: TextStyle(fontWeight: FontWeight.bold)),
                ),
              ],
            ),
          ],
        ),
      );
    }

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
