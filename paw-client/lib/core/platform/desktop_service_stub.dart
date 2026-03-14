/// Web/mobile-safe fallback for desktop-only shell hooks.
class DesktopService {
  bool get isDesktop => false;

  void setupSystemTray() {}

  void registerKeyboardShortcuts() {}
}
