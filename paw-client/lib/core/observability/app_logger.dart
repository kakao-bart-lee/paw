import 'dart:developer' as developer;

class AppLogger {
  static void event(String name, {Map<String, Object?> data = const {}}) {
    developer.log(name, name: 'paw.client', error: data.isEmpty ? null : data);
  }
}
