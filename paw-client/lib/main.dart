import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'core/di/service_locator.dart';
import 'core/platform/desktop_service.dart';
import 'core/router/app_router.dart';
import 'core/theme/app_theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await initializeDateFormatting('ko_KR');

  final desktop = DesktopService();
  final supportsFlutterClient = kIsWeb || desktop.isDesktop;

  if (supportsFlutterClient) {
    await setupServiceLocator();
  }

  if (desktop.isDesktop) {
    desktop.setupSystemTray();
    desktop.registerKeyboardShortcuts();
  }

  runApp(
    ProviderScope(
      child: PawApp(flutterClientEnabled: supportsFlutterClient),
    ),
  );
}

class PawApp extends ConsumerWidget {
  const PawApp({super.key, this.flutterClientEnabled = true});

  final bool flutterClientEnabled;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (!flutterClientEnabled) {
      return const MaterialApp(
        debugShowCheckedModeBanner: false,
        home: _UnsupportedPlatformScreen(),
      );
    }

    final router = ref.watch(appRouterProvider);
    return MaterialApp.router(
      title: 'Paw',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light,
      darkTheme: AppTheme.dark,
      themeMode: ThemeMode.dark, // Dark mode by default
      routerConfig: router,
    );
  }
}

class _UnsupportedPlatformScreen extends StatelessWidget {
  const _UnsupportedPlatformScreen();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.background,
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 440),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: AppTheme.surface2,
                borderRadius: BorderRadius.circular(28),
                border: Border.all(color: AppTheme.outline),
              ),
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Container(
                      width: 64,
                      height: 64,
                      decoration: BoxDecoration(
                        color: AppTheme.primarySoft,
                        borderRadius: BorderRadius.circular(22),
                      ),
                      child: const Icon(
                        Icons.phone_android_rounded,
                        color: AppTheme.primary,
                      ),
                    ),
                    const SizedBox(height: 18),
                    Text(
                      '모바일 앱은 네이티브로 이전되었습니다',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.titleLarge,
                    ),
                    const SizedBox(height: 10),
                    Text(
                      '이 Flutter 클라이언트는 이제 Web/Desktop 전용입니다. '
                      '모바일에서는 새 Android/iOS 앱을 사용하세요.',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodyMedium,
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
