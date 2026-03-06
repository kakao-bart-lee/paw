import 'package:flutter/foundation.dart';

/// Service for web-specific platform features.
///
/// Provides WebSocket URL conversion, web platform detection,
/// and service worker support stubs for PWA functionality.
class WebService {
  /// Whether the app is currently running on the web platform.
  bool get isWeb => kIsWeb;

  /// Converts an HTTP(S) base URL to the corresponding WebSocket URL.
  ///
  /// - `http://` → `ws://`
  /// - `https://` → `wss://`
  ///
  /// If the scheme is already `ws` or `wss`, it is returned unchanged.
  /// Unknown schemes default to `ws://`.
  String getWebSocketUrl(String baseUrl) {
    final uri = Uri.parse(baseUrl);
    final String wsScheme;
    switch (uri.scheme) {
      case 'https':
      case 'wss':
        wsScheme = 'wss';
      case 'http':
      case 'ws':
        wsScheme = 'ws';
      default:
        wsScheme = 'ws';
    }
    return uri.replace(scheme: wsScheme).toString();
  }

  /// Whether the browser supports service workers.
  ///
  /// Currently a stub — returns `true` when running on web, `false` otherwise.
  /// A real implementation would check `navigator.serviceWorker` availability.
  bool supportsServiceWorker() => kIsWeb;
}
