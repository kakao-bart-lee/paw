import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../features/auth/screens/login_screen.dart';
import '../../features/auth/screens/phone_input_screen.dart';
import '../../features/auth/screens/otp_verify_screen.dart';
import '../../features/auth/screens/device_name_screen.dart';
import '../../features/chat/screens/conversations_screen.dart';
import '../../features/chat/screens/chat_screen.dart';
import '../../features/agent/screens/agent_screen.dart';
import '../../features/settings/screens/settings_screen.dart';
import '../shell/main_shell.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/chat',
    routes: [
      // Auth routes
      GoRoute(
        path: '/login',
        builder: (context, state) => const LoginScreen(),
      ),
      GoRoute(
        path: '/auth/phone',
        builder: (context, state) => const PhoneInputScreen(),
      ),
      GoRoute(
        path: '/auth/otp',
        builder: (context, state) => const OtpVerifyScreen(),
      ),
      GoRoute(
        path: '/auth/device-name',
        builder: (context, state) => const DeviceNameScreen(),
      ),
      
      // Main shell with bottom navigation
      ShellRoute(
        builder: (context, state, child) => MainShell(child: child),
        routes: [
          GoRoute(
            path: '/chat',
            builder: (context, state) => const ConversationsScreen(),
            routes: [
              GoRoute(
                path: ':conversationId',
                builder: (context, state) => ChatScreen(
                  conversationId: state.pathParameters['conversationId']!,
                ),
              ),
            ],
          ),
          GoRoute(
            path: '/agent',
            builder: (context, state) => const AgentScreen(),
          ),
          GoRoute(
            path: '/settings',
            builder: (context, state) => const SettingsScreen(),
          ),
        ],
      ),
    ],
  );
});
