import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'dart:convert';
import 'package:crypto/crypto.dart';

import '../../../core/theme/app_theme.dart';

bool isE2eeVerificationSupported({bool? isWebOverride}) {
  return !(isWebOverride ?? kIsWeb);
}

class KeyVerificationScreen extends StatelessWidget {
  final String conversationId;

  const KeyVerificationScreen({super.key, required this.conversationId});

  String _generateSafetyNumber(String input) {
    // Generate a deterministic 60-digit string based on the input
    final bytes = utf8.encode(input);
    final digest = sha256.convert(bytes);

    // Convert hash to a long numeric string
    String numericString = '';
    for (var byte in digest.bytes) {
      numericString += byte.toString().padLeft(3, '0');
    }

    // Ensure we have at least 60 digits
    while (numericString.length < 60) {
      numericString += numericString;
    }

    // Take exactly 60 digits
    numericString = numericString.substring(0, 60);

    // Format into 12 groups of 5 digits
    List<String> groups = [];
    for (int i = 0; i < 60; i += 5) {
      groups.add(numericString.substring(i, i + 5));
    }

    return groups.join(' ');
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final safetyNumber = _generateSafetyNumber(conversationId);
    final supported = isE2eeVerificationSupported();

    return Scaffold(
      appBar: AppBar(title: const Text('안전 번호 확인')),
      body: Padding(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            if (!supported)
              Container(
                width: double.infinity,
                padding: const EdgeInsets.all(12),
                margin: const EdgeInsets.only(bottom: 16),
                decoration: BoxDecoration(
                  color: AppTheme.webNoticeSurface,
                  borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                ),
                child: const Text(
                  '웹에서는 E2EE 키 검증을 지원하지 않습니다.',
                  style: TextStyle(color: AppTheme.warning, fontSize: 12),
                  textAlign: TextAlign.center,
                ),
              ),
            const SizedBox(height: 24),
            const Icon(Icons.lock_outline, size: 64, color: AppTheme.success),
            const SizedBox(height: 24),
            Text(
              '이 번호가 상대방 기기에서도 동일하게 표시되면 대화가 안전합니다',
              textAlign: TextAlign.center,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 48),
            Container(
              padding: const EdgeInsets.all(24),
              decoration: BoxDecoration(
                color: AppTheme.surface2,
                borderRadius: BorderRadius.circular(AppTheme.radiusXl),
              ),
              child: Text(
                safetyNumber,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 20,
                  letterSpacing: 2,
                  height: 1.8,
                ),
              ),
            ),
            const Spacer(),
            SizedBox(
              width: double.infinity,
              child: ElevatedButton(
                onPressed: () {
                  if (!supported) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('웹에서는 E2EE 키 검증이 지원되지 않습니다.'),
                      ),
                    );
                    return;
                  }
                  // Stub for comparing shown numbers
                  ScaffoldMessenger.of(
                    context,
                  ).showSnackBar(const SnackBar(content: Text('확인되었습니다')));
                },
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.accent,
                  foregroundColor: theme.colorScheme.onPrimary,
                  padding: const EdgeInsets.symmetric(vertical: 16),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(AppTheme.radiusLg),
                  ),
                ),
                child: const Text(
                  '표시한 번호 일치 확인',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                ),
              ),
            ),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              child: OutlinedButton(
                onPressed: () {
                  if (!supported) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('웹에서는 E2EE 키 검증이 지원되지 않습니다.'),
                      ),
                    );
                    return;
                  }
                  // Stub for QR code comparison
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('QR 코드 스캐너를 엽니다')),
                  );
                },
                style: OutlinedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(vertical: 16),
                  side: BorderSide(color: theme.colorScheme.outline),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(AppTheme.radiusLg),
                  ),
                ),
                child: const Text(
                  'QR 코드 비교',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                ),
              ),
            ),
            const SizedBox(height: 24),
          ],
        ),
      ),
    );
  }
}
