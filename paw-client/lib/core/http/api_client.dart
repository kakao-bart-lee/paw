import 'dart:convert';

import 'package:http/http.dart' as http;

class ApiClient {
  final String baseUrl;
  String? _accessToken;

  ApiClient({required this.baseUrl});

  void setToken(String token) => _accessToken = token;

  String? get accessToken => _accessToken;

  Map<String, String> get _headers => {
    'Content-Type': 'application/json',
    if (_accessToken != null) 'Authorization': 'Bearer $_accessToken',
  };

  // Auth
  Future<Map<String, dynamic>> requestOtp(String phone) async {
    final response = await _post('/auth/request-otp', body: {'phone': phone});
    return _decodeJsonObject(response);
  }

  Future<Map<String, dynamic>> verifyOtp(String phone, String code) async {
    final response = await _post(
      '/auth/verify-otp',
      body: {'phone': phone, 'code': code},
    );
    return _decodeJsonObject(response);
  }

  Future<Map<String, dynamic>> registerDevice(
    String sessionToken,
    String deviceName,
    String ed25519PubKeyBase64,
  ) async {
    final response = await _post(
      '/auth/register-device',
      body: {
        'session_token': sessionToken,
        'device_name': deviceName,
        'ed25519_public_key': ed25519PubKeyBase64,
      },
    );
    return _decodeJsonObject(response);
  }

  Future<Map<String, dynamic>> refreshToken(String refreshToken) async {
    final response = await _post(
      '/auth/refresh',
      body: {'refresh_token': refreshToken},
    );
    return _decodeJsonObject(response);
  }

  // Conversations
  Future<List<Map<String, dynamic>>> getConversations() async {
    final response = await _get('/conversations');
    final json = _decodeJsonObject(response);
    final list = (json['conversations'] as List?) ?? const [];
    return list
        .whereType<Map>()
        .map((item) => Map<String, dynamic>.from(item))
        .toList();
  }

  Future<Map<String, dynamic>> createConversation(
    List<String> memberIds, {
    String? name,
  }) async {
    final body = <String, dynamic>{
      'member_ids': memberIds,
      if (name != null && name.trim().isNotEmpty) 'name': name.trim(),
    };

    final response = await _post('/conversations', body: body);
    return _decodeJsonObject(response);
  }

  Future<void> addMember(String convId, String userId) async {
    await _post('/conversations/$convId/members', body: {'user_id': userId});
  }

  Future<void> removeMember(String convId, String userId) async {
    final uri = _buildUri('/conversations/$convId/members/$userId');
    final response = await http.delete(uri, headers: _headers);
    _throwIfError(response);
  }

  // Messages
  Future<Map<String, dynamic>> sendMessage(
    String convId,
    String content,
    String idempotencyKey,
  ) async {
    final response = await _post(
      '/conversations/$convId/messages',
      body: {
        'content': content,
        'format': 'plain',
        'idempotency_key': idempotencyKey,
      },
    );
    return _decodeJsonObject(response);
  }

  Future<Map<String, dynamic>> getMessages(
    String convId, {
    int afterSeq = 0,
    int limit = 50,
  }) async {
    final response = await _get(
      '/conversations/$convId/messages',
      queryParameters: {
        'after_seq': '$afterSeq',
        'limit': '$limit',
      },
    );
    return _decodeJsonObject(response);
  }

  // Users
  Future<Map<String, dynamic>> getMe() async {
    final response = await _get('/users/me');
    return _decodeJsonObject(response);
  }

  Future<Map<String, dynamic>> updateMe({
    String? displayName,
    String? avatarUrl,
  }) async {
    final response = await _patch(
      '/users/me',
      body: {
        'display_name': displayName,
        'avatar_url': avatarUrl,
      },
    );
    return _decodeJsonObject(response);
  }

  Future<Map<String, dynamic>?> searchUser(String phone) async {
    final uri = _buildUri('/users/search', queryParameters: {'phone': phone});
    final response = await http.get(uri, headers: _headers);

    if (response.statusCode == 404) {
      return null;
    }

    _throwIfError(response);
    return _decodeJsonObject(response);
  }

  Future<void> uploadKeyBundle(Map<String, dynamic> bundle) async {
    await _post('/api/v1/keys/bundle', body: bundle);
  }

  Future<Map<String, dynamic>?> getKeyBundle(String userId) async {
    try {
      final response = await _get('/api/v1/keys/$userId');
      return _decodeJsonObject(response);
    } on ApiException catch (e) {
      if (e.statusCode == 404) return null;
      rethrow;
    }
  }

  Future<http.Response> _get(
    String path, {
    Map<String, String>? queryParameters,
  }) async {
    final uri = _buildUri(path, queryParameters: queryParameters);
    final response = await http.get(uri, headers: _headers);
    _throwIfError(response);
    return response;
  }

  Future<http.Response> _post(
    String path, {
    required Map<String, dynamic> body,
  }) async {
    final uri = _buildUri(path);
    final response = await http.post(
      uri,
      headers: _headers,
      body: jsonEncode(body),
    );
    _throwIfError(response);
    return response;
  }

  Future<http.Response> _patch(
    String path, {
    required Map<String, dynamic> body,
  }) async {
    final uri = _buildUri(path);
    final response = await http.patch(
      uri,
      headers: _headers,
      body: jsonEncode(body),
    );
    _throwIfError(response);
    return response;
  }

  Uri _buildUri(String path, {Map<String, String>? queryParameters}) {
    final base = Uri.parse(baseUrl);
    final normalizedPath = path.startsWith('/') ? path : '/$path';
    return base.replace(
      path: '${base.path}$normalizedPath'.replaceAll('//', '/'),
      queryParameters: queryParameters,
    );
  }

  Map<String, dynamic> _decodeJsonObject(http.Response response) {
    if (response.body.isEmpty) {
      return <String, dynamic>{};
    }

    final decoded = jsonDecode(response.body);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('Expected JSON object response');
    }
    return decoded;
  }

  void _throwIfError(http.Response response) {
    if (response.statusCode >= 200 && response.statusCode < 300) {
      return;
    }

    String message = 'HTTP ${response.statusCode}';

    if (response.body.isNotEmpty) {
      try {
        final decoded = jsonDecode(response.body);
        if (decoded is Map<String, dynamic>) {
          message = (decoded['message'] as String?) ??
              (decoded['error'] as String?) ??
              message;
        }
      } catch (_) {
        // Ignore parse failures and keep fallback message.
      }
    }

    throw ApiException(response.statusCode, message);
  }
}

class ApiException implements Exception {
  final int statusCode;
  final String message;

  ApiException(this.statusCode, this.message);

  @override
  String toString() => 'ApiException($statusCode): $message';
}
