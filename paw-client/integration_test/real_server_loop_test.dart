import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:http/http.dart' as http;
import 'package:integration_test/integration_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:uuid/uuid.dart';

import 'package:paw_client/core/di/service_locator.dart';
import 'package:paw_client/core/router/app_router.dart';
import 'package:paw_client/features/auth/providers/auth_provider.dart';
import 'package:paw_client/features/chat/providers/chat_provider.dart';

const _uuid = Uuid();
const _apiBaseUrl = String.fromEnvironment(
  'SERVER_URL',
  defaultValue: 'http://127.0.0.1:3000',
);

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await initializeDateFormatting('ko_KR');
    await setupServiceLocator();
  });

  testWidgets(
    'real-server loop: login -> list -> send/receive -> restart restore',
    (tester) async {
      final phone = _uniquePhone();
      final conversationTitle =
          'real-e2e-${DateTime.now().millisecondsSinceEpoch}';
      const outgoing = 'e2e-outgoing';
      const incoming = 'e2e-incoming';

      final container = ProviderContainer();
      addTearDown(container.dispose);

      await tester.pumpWidget(_app(container));
      await tester.pump();

      await _waitFor(tester, find.byKey(const ValueKey('phone-input')));

      await tester.enterText(
        find.byKey(const ValueKey('phone-input')),
        _localPhoneDigits(phone),
      );
      await tester.pump();

      await container.read(authNotifierProvider.notifier).requestOtp(phone);
      final router = container.read(appRouterProvider);
      router.go('/auth/otp');
      await tester.pump();

      final otpCode = await _requestOtpAndGetDebugCode(phone);
      await container.read(authNotifierProvider.notifier).verifyOtp(otpCode);
      final afterVerify = container.read(authNotifierProvider);
      if (afterVerify.step != AuthStep.deviceName) {
        throw TestFailure(
          'verifyOtp did not reach deviceName. step=${afterVerify.step} error=${afterVerify.error}',
        );
      }
      await container
          .read(authNotifierProvider.notifier)
          .setDeviceName('real-e2e-device');
      final afterSetDevice = container.read(authNotifierProvider);
      if (afterSetDevice.step != AuthStep.authenticated) {
        throw TestFailure(
          'setDeviceName did not authenticate. step=${afterSetDevice.step} error=${afterSetDevice.error}',
        );
      }
      router.go('/chat');
      await tester.pump();

      await _waitFor(tester, find.text('채팅'));

      final token = container.read(authNotifierProvider).accessToken;
      expect(token, isNotNull);

      final conversationId = await _createConversation(
        accessToken: token!,
        title: conversationTitle,
      );

      await container.read(conversationsNotifierProvider.notifier).refresh();
      await tester.pump();
      await _waitFor(tester, find.text(conversationTitle));

      await tester.tap(find.text(conversationTitle).first);
      await tester.pump();

      await _waitFor(tester, find.byKey(const ValueKey('chat-message-input')));
      final sendResult = await container
          .read(messagesNotifierProvider(conversationId).notifier)
          .sendMessage(outgoing);
      if (!sendResult.ok) {
        throw TestFailure('sendMessage failed: ${sendResult.message}');
      }
      await tester.pump();

      await _waitFor(tester, find.text(outgoing));

      await _sendServerMessage(
        accessToken: token,
        conversationId: conversationId,
        content: incoming,
      );

      await _waitFor(
        tester,
        find.text(incoming),
        timeout: const Duration(seconds: 15),
      );

      // Restart app and verify native session restore + message history reload.
      final container2 = ProviderContainer();
      addTearDown(container2.dispose);

      await tester.pumpWidget(_app(container2));
      await tester.pump();

      await _waitFor(tester, find.text('채팅'));
      await container2.read(conversationsNotifierProvider.notifier).refresh();
      await tester.pump();
      await _waitFor(tester, find.text(conversationTitle));

      await tester.tap(find.text(conversationTitle).first);
      await tester.pump();

      await _waitFor(tester, find.text(outgoing));
      await _waitFor(tester, find.text(incoming));
    },
  );
}

Widget _app(ProviderContainer container) {
  return UncontrolledProviderScope(
    container: container,
    child: Consumer(
      builder: (context, ref, _) {
        final router = ref.watch(appRouterProvider);
        return MaterialApp.router(routerConfig: router);
      },
    ),
  );
}

Future<void> _waitFor(
  WidgetTester tester,
  Finder finder, {
  Duration timeout = const Duration(seconds: 10),
}) async {
  final end = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(end)) {
    await tester.pump(const Duration(milliseconds: 200));
    if (finder.evaluate().isNotEmpty) {
      return;
    }
  }
  throw TestFailure('Timed out waiting for finder: $finder');
}

String _uniquePhone() {
  final suffix = '${DateTime.now().millisecondsSinceEpoch}'.substring(5);
  return '+8210$suffix';
}

String _localPhoneDigits(String phone) {
  return phone.replaceFirst('+82', '');
}

Future<String> _requestOtpAndGetDebugCode(String phone) async {
  final response = await http.post(
    Uri.parse('$_apiBaseUrl/auth/request-otp'),
    headers: {'Content-Type': 'application/json'},
    body: jsonEncode({'phone': phone}),
  );

  if (response.statusCode < 200 || response.statusCode >= 300) {
    throw TestFailure(
      'requestOtp failed: ${response.statusCode} ${response.body}',
    );
  }

  final json = jsonDecode(response.body) as Map<String, dynamic>;
  final code = json['debug_code'] as String?;
  if (code == null || code.length != 6) {
    throw TestFailure(
      'debug_code missing. Set PAW_EXPOSE_OTP_FOR_E2E=true on server. body=${response.body}',
    );
  }
  return code;
}

Future<String> _createConversation({
  required String accessToken,
  required String title,
}) async {
  final response = await http.post(
    Uri.parse('$_apiBaseUrl/conversations'),
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer $accessToken',
    },
    body: jsonEncode({'member_ids': <String>[], 'name': title}),
  );

  if (response.statusCode < 200 || response.statusCode >= 300) {
    throw TestFailure(
      'createConversation failed: ${response.statusCode} ${response.body}',
    );
  }

  final json = jsonDecode(response.body) as Map<String, dynamic>;
  final id = json['id'] as String?;
  if (id == null || id.isEmpty) {
    throw TestFailure('conversation id missing: ${response.body}');
  }
  return id;
}

Future<void> _sendServerMessage({
  required String accessToken,
  required String conversationId,
  required String content,
}) async {
  final response = await http.post(
    Uri.parse('$_apiBaseUrl/conversations/$conversationId/messages'),
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer $accessToken',
    },
    body: jsonEncode({
      'content': content,
      'format': 'plain',
      'idempotency_key': _uuid.v4(),
    }),
  );

  if (response.statusCode < 200 || response.statusCode >= 300) {
    throw TestFailure(
      'sendMessage failed: ${response.statusCode} ${response.body}',
    );
  }
}
