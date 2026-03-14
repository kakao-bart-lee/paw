import 'dart:io' show Platform;

import 'package:flutter/foundation.dart';

/// Service for desktop-specific platform features.
///
/// Provides system tray integration, keyboard shortcuts, and
/// platform detection for macOS, Windows, and Linux.
class DesktopService {
  /// Whether the current platform is a desktop OS.
  bool get isDesktop =>
      !kIsWeb &&
      (Platform.isMacOS || Platform.isWindows || Platform.isLinux);

  /// Sets up the system tray icon and menu.
  ///
  /// Currently a stub — no real tray SDK is integrated.
  void setupSystemTray() {
    if (!isDesktop) return;
    debugPrint('[DesktopService] system tray setup');
  }

  /// Registers global keyboard shortcuts for the desktop app.
  ///
  /// Currently a stub — logs registration without binding real shortcuts.
  void registerKeyboardShortcuts() {
    if (!isDesktop) return;
    debugPrint('[DesktopService] keyboard shortcuts registered');
  }
}
