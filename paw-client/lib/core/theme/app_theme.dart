import 'package:flutter/material.dart';

class AppTheme {
  AppTheme._();

  static const Color background = Color(0xFF0B1113);
  static const Color surface1 = Color(0xFF10181B);
  static const Color surface2 = Color(0xFF141F23);
  static const Color surface3 = Color(0xFF1A262B);
  static const Color surface4 = Color(0xFF223137);
  static const Color outline = Color(0xFF2A3C43);
  static const Color primary = Color(0xFF63E6BE);
  static const Color primaryDeep = Color(0xFF1E7F67);
  static const Color primarySoft = Color(0xFF15332C);
  static const Color accent = Color(0xFF8EC5FF);
  static const Color online = Color(0xFF44D17A);
  static const Color warning = Color(0xFFF5B642);
  static const Color danger = Color(0xFFFF7A7A);
  static const Color mutedText = Color(0xFF94A8AF);
  static const Color strongText = Color(0xFFF5FAFC);

  static const Color sentBubbleDark = Color(0xFF1B7D66);
  static const Color receivedBubbleDark = surface2;
  static const Color agentBubbleDark = Color(0xFF11262B);

  static ThemeData get dark {
    const colorScheme = ColorScheme.dark(
      primary: primary,
      onPrimary: Color(0xFF06211A),
      secondary: accent,
      onSecondary: Color(0xFF08161F),
      surface: surface1,
      onSurface: strongText,
      onSurfaceVariant: mutedText,
      outline: outline,
      error: danger,
      onError: Colors.white,
    );

    final base = ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      colorScheme: colorScheme,
      scaffoldBackgroundColor: background,
      canvasColor: background,
      splashFactory: InkRipple.splashFactory,
    );

    final textTheme = base.textTheme.copyWith(
      headlineLarge: base.textTheme.headlineLarge?.copyWith(
        fontSize: 30,
        fontWeight: FontWeight.w800,
        color: strongText,
        letterSpacing: -0.8,
      ),
      headlineMedium: base.textTheme.headlineMedium?.copyWith(
        fontSize: 24,
        fontWeight: FontWeight.w700,
        color: strongText,
        letterSpacing: -0.6,
      ),
      titleLarge: base.textTheme.titleLarge?.copyWith(
        fontSize: 20,
        fontWeight: FontWeight.w700,
        color: strongText,
        letterSpacing: -0.4,
      ),
      titleMedium: base.textTheme.titleMedium?.copyWith(
        fontSize: 16,
        fontWeight: FontWeight.w600,
        color: strongText,
      ),
      bodyLarge: base.textTheme.bodyLarge?.copyWith(
        fontSize: 15,
        height: 1.45,
        color: strongText,
      ),
      bodyMedium: base.textTheme.bodyMedium?.copyWith(
        fontSize: 14,
        height: 1.45,
        color: strongText,
      ),
      bodySmall: base.textTheme.bodySmall?.copyWith(
        fontSize: 12,
        height: 1.35,
        color: mutedText,
      ),
      labelLarge: base.textTheme.labelLarge?.copyWith(
        fontSize: 13,
        fontWeight: FontWeight.w600,
      ),
      labelMedium: base.textTheme.labelMedium?.copyWith(
        fontSize: 12,
        fontWeight: FontWeight.w600,
      ),
      labelSmall: base.textTheme.labelSmall?.copyWith(
        fontSize: 11,
        fontWeight: FontWeight.w600,
        color: mutedText,
      ),
    );

    return base.copyWith(
      textTheme: textTheme,
      appBarTheme: const AppBarTheme(
        backgroundColor: Colors.transparent,
        foregroundColor: strongText,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: false,
        surfaceTintColor: Colors.transparent,
      ),
      navigationBarTheme: NavigationBarThemeData(
        backgroundColor: surface2,
        height: 74,
        indicatorColor: primarySoft,
        surfaceTintColor: Colors.transparent,
        labelTextStyle: const MaterialStatePropertyAll(
          TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
        ),
        iconTheme: MaterialStateProperty.resolveWith((states) {
          final isSelected = states.contains(MaterialState.selected);
          return IconThemeData(
            size: 22,
            color: isSelected ? primary : mutedText,
          );
        }),
      ),
      dividerTheme: const DividerThemeData(
        color: outline,
        thickness: 1,
        space: 1,
      ),
      cardTheme: CardThemeData(
        color: surface2,
        elevation: 0,
        surfaceTintColor: Colors.transparent,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(24),
          side: const BorderSide(color: outline),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: surface4,
        contentTextStyle: textTheme.bodyMedium?.copyWith(color: strongText),
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(18)),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: surface3,
        hintStyle: textTheme.bodyMedium?.copyWith(color: mutedText),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: 18,
          vertical: 14,
        ),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(20),
          borderSide: const BorderSide(color: outline),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(20),
          borderSide: const BorderSide(color: outline),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(20),
          borderSide: const BorderSide(color: primary, width: 1.2),
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: primary,
          foregroundColor: colorScheme.onPrimary,
          elevation: 0,
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(18),
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: strongText,
          side: const BorderSide(color: outline),
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(18),
          ),
        ),
      ),
      listTileTheme: const ListTileThemeData(
        contentPadding: EdgeInsets.symmetric(horizontal: 16, vertical: 4),
        iconColor: mutedText,
      ),
      chipTheme: base.chipTheme.copyWith(
        backgroundColor: surface3,
        selectedColor: primarySoft,
        side: const BorderSide(color: outline),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
        labelStyle: textTheme.labelMedium,
      ),
    );
  }

  static ThemeData get light {
    final scheme = ColorScheme.fromSeed(
      seedColor: primaryDeep,
      brightness: Brightness.light,
    );
    return ThemeData(
      useMaterial3: true,
      colorScheme: scheme,
      scaffoldBackgroundColor: const Color(0xFFF6FBFC),
      splashFactory: InkRipple.splashFactory,
    );
  }
}
