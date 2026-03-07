import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../features/auth/providers/auth_provider.dart';
import '../../features/auth/screens/login_screen.dart';
import '../../features/auth/screens/phone_input_screen.dart';
import '../../features/auth/screens/otp_verify_screen.dart';
import '../../features/auth/screens/device_name_screen.dart';
import '../../features/chat/screens/conversations_screen.dart';
import '../../features/chat/screens/chat_screen.dart';
import '../../features/chat/screens/key_verification_screen.dart';
import '../../features/chat/screens/group_info_screen.dart';
import '../../features/chat/screens/create_group_screen.dart';
import '../../features/chat/screens/search_screen.dart';
import '../../features/agent/screens/agent_screen.dart';
import '../../features/settings/screens/settings_screen.dart';
import '../../features/profile/screens/my_profile_screen.dart';
import '../../features/profile/screens/user_profile_screen.dart';
import '../shell/main_shell.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  final authState = ref.watch(authNotifierProvider);

  bool isPublicPath(String path) {
    return path == '/login' || path.startsWith('/auth/');
  }

  return GoRouter(
    initialLocation: '/chat',
    redirect: (context, state) {
      final path = state.uri.path;
      final isAuthenticated = authState.step == AuthStep.authenticated;

      if (!isAuthenticated && !isPublicPath(path)) {
        return '/auth/phone';
      }

      if (isAuthenticated && isPublicPath(path)) {
        return '/chat';
      }

      return null;
    },
    routes: [
      // Auth routes
      GoRoute(path: '/login', builder: (context, state) => const LoginScreen()),
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

      // Search route (outside shell — no bottom nav)
      GoRoute(
        path: '/search',
        builder: (context, state) => const SearchScreen(),
      ),

      // Profile routes (outside shell — no bottom nav)
      GoRoute(
        path: '/group/:id/info',
        builder: (context, state) =>
            GroupInfoScreen(conversationId: state.pathParameters['id']!),
      ),
      GoRoute(
        path: '/create-group',
        builder: (context, state) => const CreateGroupScreen(),
      ),
      GoRoute(
        path: '/profile/me',
        builder: (context, state) => const MyProfileScreen(),
      ),
      GoRoute(
        path: '/profile/:userId',
        builder: (context, state) =>
            UserProfileScreen(userId: state.pathParameters['userId']!),
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
              GoRoute(
                path: ':id/verify',
                builder: (context, state) => KeyVerificationScreen(
                  conversationId: state.pathParameters['id']!,
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
