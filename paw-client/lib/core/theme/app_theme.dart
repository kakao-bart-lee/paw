import 'package:flutter/material.dart';

class AppTheme {
  AppTheme._();

  static const Color background = Color(0xFF0C0D0B);
  static const Color surface1 = Color(0xFF10110F);
  static const Color surface2 = Color(0xFF131412);
  static const Color surface3 = Color(0xFF1A1A19);
  static const Color surface4 = Color(0xFF1F1F1E);
  static const Color outline = Color(0xFF262624);
  static const Color primary = Color(0xFFF0EBE0);
  static const Color primaryDeep = Color(0xFFD8D1C5);
  static const Color primarySoft = Color(0xFF1B1A18);
  static const Color accent = Color(0xFFC8832A);
  static const Color online = Color(0xFF44D17A);
  static const Color warning = Color(0xFFF5B642);
  static const Color danger = Color(0xFF902020);
  static const Color mutedText = Color(0xFF9A9086);
  static const Color strongText = Color(0xFFF0EBE0);

  static const Color sentBubbleDark = Color(0xFF201E19);
  static const Color receivedBubbleDark = surface2;
  static const Color agentBubbleDark = Color(0xFF171715);

  static ThemeData get dark {
    const colorScheme = ColorScheme.dark(
      primary: primary,
      onPrimary: background,
      secondary: accent,
      onSecondary: background,
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

    TextStyle mono(TextStyle? style) => (style ?? const TextStyle()).copyWith(
      fontFamilyFallback: const [
        'IBM Plex Mono',
        'ui-monospace',
        'SFMono-Regular',
        'Menlo',
        'monospace',
      ],
    );

    final textTheme = base.textTheme.copyWith(
      headlineLarge: mono(base.textTheme.headlineLarge).copyWith(
        fontSize: 30,
        fontWeight: FontWeight.w700,
        color: strongText,
        letterSpacing: -0.4,
      ),
      headlineMedium: mono(base.textTheme.headlineMedium).copyWith(
        fontSize: 24,
        fontWeight: FontWeight.w700,
        color: strongText,
        letterSpacing: -0.2,
      ),
      titleLarge: mono(base.textTheme.titleLarge).copyWith(
        fontSize: 20,
        fontWeight: FontWeight.w700,
        color: strongText,
        letterSpacing: -0.1,
      ),
      titleMedium: mono(
        base.textTheme.titleMedium,
      ).copyWith(fontSize: 16, fontWeight: FontWeight.w600, color: strongText),
      bodyLarge: mono(
        base.textTheme.bodyLarge,
      ).copyWith(fontSize: 15, height: 1.55, color: strongText),
      bodyMedium: mono(
        base.textTheme.bodyMedium,
      ).copyWith(fontSize: 14, height: 1.55, color: strongText),
      bodySmall: mono(
        base.textTheme.bodySmall,
      ).copyWith(fontSize: 12, height: 1.45, color: mutedText),
      labelLarge: mono(
        base.textTheme.labelLarge,
      ).copyWith(fontSize: 13, fontWeight: FontWeight.w600),
      labelMedium: mono(
        base.textTheme.labelMedium,
      ).copyWith(fontSize: 12, fontWeight: FontWeight.w600),
      labelSmall: mono(
        base.textTheme.labelSmall,
      ).copyWith(fontSize: 11, fontWeight: FontWeight.w600, color: mutedText),
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
        labelTextStyle: const WidgetStatePropertyAll(
          TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
        ),
        iconTheme: WidgetStateProperty.resolveWith((states) {
          final isSelected = states.contains(WidgetState.selected);
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
          borderRadius: BorderRadius.circular(10),
          side: const BorderSide(color: outline),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: surface4,
        contentTextStyle: textTheme.bodyMedium?.copyWith(color: strongText),
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
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
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: outline),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: outline),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: accent, width: 1.2),
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: accent,
          foregroundColor: colorScheme.onPrimary,
          elevation: 0,
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          backgroundColor: accent,
          foregroundColor: colorScheme.onPrimary,
          disabledBackgroundColor: surface4,
          disabledForegroundColor: mutedText,
          elevation: 0,
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: strongText,
          side: const BorderSide(color: outline),
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      listTileTheme: const ListTileThemeData(
        contentPadding: EdgeInsets.symmetric(horizontal: 16, vertical: 4),
        iconColor: mutedText,
      ),
      floatingActionButtonTheme: const FloatingActionButtonThemeData(
        backgroundColor: accent,
        foregroundColor: background,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.all(Radius.circular(8)),
        ),
      ),
      chipTheme: base.chipTheme.copyWith(
        backgroundColor: surface3,
        selectedColor: primarySoft,
        side: const BorderSide(color: outline),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        labelStyle: textTheme.labelMedium,
      ),
    );
  }

  static ThemeData get light {
    const lightBackground = Color(0xFFF5F0E8);
    const lightSurface = Color(0xFFF8F5F0);
    const lightSurfaceAlt = Color(0xFFEBE5D9);
    const lightOutline = Color(0xFFDDD7CC);
    const lightStrongText = Color(0xFF2C2A25);
    const lightMutedText = Color(0xFF7A7570);

    const scheme = ColorScheme.light(
      primary: lightStrongText,
      onPrimary: lightBackground,
      secondary: lightSurfaceAlt,
      onSecondary: lightStrongText,
      surface: lightSurface,
      onSurface: lightStrongText,
      onSurfaceVariant: lightMutedText,
      outline: lightOutline,
      error: Color(0xFFE03030),
      onError: lightBackground,
    );

    TextStyle mono(TextStyle? style) => (style ?? const TextStyle()).copyWith(
      fontFamilyFallback: const [
        'IBM Plex Mono',
        'ui-monospace',
        'SFMono-Regular',
        'Menlo',
        'monospace',
      ],
    );

    final base = ThemeData(
      useMaterial3: true,
      colorScheme: scheme,
      scaffoldBackgroundColor: lightBackground,
      splashFactory: InkRipple.splashFactory,
    );

    final textTheme = base.textTheme.copyWith(
      headlineLarge: mono(base.textTheme.headlineLarge).copyWith(
        fontSize: 30,
        fontWeight: FontWeight.w700,
        color: lightStrongText,
        letterSpacing: -0.4,
      ),
      headlineMedium: mono(base.textTheme.headlineMedium).copyWith(
        fontSize: 24,
        fontWeight: FontWeight.w700,
        color: lightStrongText,
        letterSpacing: -0.2,
      ),
      titleLarge: mono(base.textTheme.titleLarge).copyWith(
        fontSize: 20,
        fontWeight: FontWeight.w700,
        color: lightStrongText,
      ),
      titleMedium: mono(base.textTheme.titleMedium).copyWith(
        fontSize: 16,
        fontWeight: FontWeight.w600,
        color: lightStrongText,
      ),
      bodyLarge: mono(
        base.textTheme.bodyLarge,
      ).copyWith(fontSize: 15, height: 1.55, color: lightStrongText),
      bodyMedium: mono(
        base.textTheme.bodyMedium,
      ).copyWith(fontSize: 14, height: 1.55, color: lightStrongText),
      bodySmall: mono(
        base.textTheme.bodySmall,
      ).copyWith(fontSize: 12, height: 1.45, color: lightMutedText),
      labelLarge: mono(base.textTheme.labelLarge).copyWith(
        fontSize: 13,
        fontWeight: FontWeight.w600,
        color: lightStrongText,
      ),
      labelMedium: mono(base.textTheme.labelMedium).copyWith(
        fontSize: 12,
        fontWeight: FontWeight.w600,
        color: lightStrongText,
      ),
      labelSmall: mono(base.textTheme.labelSmall).copyWith(
        fontSize: 11,
        fontWeight: FontWeight.w600,
        color: lightMutedText,
      ),
    );

    return base.copyWith(
      textTheme: textTheme,
      appBarTheme: const AppBarTheme(
        backgroundColor: Colors.transparent,
        foregroundColor: lightStrongText,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: false,
        surfaceTintColor: Colors.transparent,
      ),
      navigationBarTheme: NavigationBarThemeData(
        backgroundColor: lightSurface,
        height: 74,
        indicatorColor: lightSurfaceAlt,
        surfaceTintColor: Colors.transparent,
        labelTextStyle: const WidgetStatePropertyAll(
          TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
        ),
        iconTheme: WidgetStateProperty.resolveWith((states) {
          final isSelected = states.contains(WidgetState.selected);
          return IconThemeData(
            size: 22,
            color: isSelected ? accent : lightMutedText,
          );
        }),
      ),
      dividerTheme: const DividerThemeData(
        color: lightOutline,
        thickness: 1,
        space: 1,
      ),
      cardTheme: CardThemeData(
        color: lightSurface,
        elevation: 0,
        surfaceTintColor: Colors.transparent,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(10),
          side: const BorderSide(color: lightOutline),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: lightSurfaceAlt,
        contentTextStyle: textTheme.bodyMedium?.copyWith(
          color: lightStrongText,
        ),
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: lightSurface,
        hintStyle: textTheme.bodyMedium?.copyWith(color: lightMutedText),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: 18,
          vertical: 14,
        ),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: lightOutline),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: lightOutline),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: accent, width: 1.2),
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: accent,
          foregroundColor: lightStrongText,
          elevation: 0,
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          backgroundColor: accent,
          foregroundColor: lightStrongText,
          disabledBackgroundColor: lightSurfaceAlt,
          disabledForegroundColor: lightMutedText,
          elevation: 0,
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: lightStrongText,
          side: const BorderSide(color: lightOutline),
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      listTileTheme: const ListTileThemeData(
        contentPadding: EdgeInsets.symmetric(horizontal: 16, vertical: 4),
        iconColor: lightMutedText,
      ),
      chipTheme: base.chipTheme.copyWith(
        backgroundColor: lightSurfaceAlt,
        selectedColor: lightSurface,
        side: const BorderSide(color: lightOutline),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        labelStyle: textTheme.labelMedium,
      ),
      floatingActionButtonTheme: const FloatingActionButtonThemeData(
        backgroundColor: accent,
        foregroundColor: lightStrongText,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.all(Radius.circular(8)),
        ),
      ),
    );
  }
}
