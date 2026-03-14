import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

class ThemeModeController extends ChangeNotifier {
  ThemeModeController();

  static const _prefsKey = 'paw.theme_mode';

  ThemeMode _themeMode = ThemeMode.dark;
  SharedPreferences? _prefs;

  ThemeMode get themeMode => _themeMode;
  bool get isDarkMode => _themeMode == ThemeMode.dark;

  Future<void> load() async {
    _prefs ??= await SharedPreferences.getInstance();
    final stored = _prefs!.getString(_prefsKey);
    switch (stored) {
      case 'light':
        _themeMode = ThemeMode.light;
      case 'dark':
        _themeMode = ThemeMode.dark;
      case 'system':
        _themeMode = ThemeMode.system;
      default:
        _themeMode = ThemeMode.dark;
    }
  }

  Future<void> setThemeMode(ThemeMode mode) async {
    if (_themeMode == mode) return;
    _themeMode = mode;
    notifyListeners();
    _prefs ??= await SharedPreferences.getInstance();
    await _prefs!.setString(_prefsKey, mode.name);
  }

  Future<void> toggleDarkMode(bool enabled) async {
    await setThemeMode(enabled ? ThemeMode.dark : ThemeMode.light);
  }
}
