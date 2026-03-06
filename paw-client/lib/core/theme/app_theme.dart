import 'package:flutter/material.dart';

class AppTheme {
  AppTheme._();

  // ─── Color Palette ────────────────────────────────────────────────────
  static const Color _primaryDark = Color(0xFF6C63FF);    // Purple accent
  static const Color _backgroundDark = Color(0xFF0F0F0F); // Near-black
  static const Color _surfaceDark = Color(0xFF1A1A1A);    // Card surface
  static const Color _surfaceVariantDark = Color(0xFF252525);
  static const Color _onSurfaceDark = Color(0xFFE8E8E8);  // Primary text
  static const Color _onSurfaceVariantDark = Color(0xFF9E9E9E); // Secondary text
  static const Color _outlineDark = Color(0xFF333333);    // Borders
  static const Color _errorDark = Color(0xFFCF6679);
  
  // Message bubble colors
  static const Color sentBubbleDark = Color(0xFF6C63FF);   // Sent: purple
  static const Color receivedBubbleDark = Color(0xFF252525); // Received: dark gray
  static const Color agentBubbleDark = Color(0xFF1E2A3A);   // Agent: dark blue-gray

  // ─── Dark Theme ───────────────────────────────────────────────────────
  static ThemeData get dark => ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: const ColorScheme.dark(
      primary: _primaryDark,
      onPrimary: Colors.white,
      secondary: Color(0xFF03DAC6),
      background: _backgroundDark,
      surface: _surfaceDark,
      surfaceVariant: _surfaceVariantDark,
      onBackground: _onSurfaceDark,
      onSurface: _onSurfaceDark,
      onSurfaceVariant: _onSurfaceVariantDark,
      outline: _outlineDark,
      error: _errorDark,
    ),
    scaffoldBackgroundColor: _backgroundDark,
    
    // AppBar
    appBarTheme: const AppBarTheme(
      backgroundColor: _backgroundDark,
      foregroundColor: _onSurfaceDark,
      elevation: 0,
      scrolledUnderElevation: 1,
      surfaceTintColor: Colors.transparent,
    ),
    
    // Bottom Navigation
    navigationBarTheme: NavigationBarThemeData(
      backgroundColor: _surfaceDark,
      indicatorColor: _primaryDark.withOpacity(0.2),
      labelTextStyle: MaterialStateProperty.all(
        const TextStyle(fontSize: 12, fontWeight: FontWeight.w500),
      ),
    ),
    
    // Cards
    cardTheme: const CardThemeData(
      color: _surfaceDark,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(12)),
      ),
    ),
    
    // Input fields
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: _surfaceVariantDark,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: BorderSide.none,
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: _primaryDark, width: 1.5),
      ),
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
    ),
    
    // Dividers
    dividerTheme: const DividerThemeData(
      color: _outlineDark,
      thickness: 0.5,
    ),
    
    // Typography
    textTheme: const TextTheme(
      headlineLarge: TextStyle(fontSize: 28, fontWeight: FontWeight.bold, color: _onSurfaceDark),
      headlineMedium: TextStyle(fontSize: 22, fontWeight: FontWeight.bold, color: _onSurfaceDark),
      titleLarge: TextStyle(fontSize: 18, fontWeight: FontWeight.w600, color: _onSurfaceDark),
      titleMedium: TextStyle(fontSize: 16, fontWeight: FontWeight.w500, color: _onSurfaceDark),
      bodyLarge: TextStyle(fontSize: 16, color: _onSurfaceDark),
      bodyMedium: TextStyle(fontSize: 14, color: _onSurfaceDark),
      bodySmall: TextStyle(fontSize: 12, color: _onSurfaceVariantDark),
      labelSmall: TextStyle(fontSize: 11, color: _onSurfaceVariantDark),
    ),
  );

  // ─── Light Theme ──────────────────────────────────────────────────────
  static ThemeData get light => ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorScheme: ColorScheme.fromSeed(
      seedColor: const Color(0xFF6C63FF),
      brightness: Brightness.light,
    ),
  );
}
