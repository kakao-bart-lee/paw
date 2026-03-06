import 'dart:convert';
import 'dart:typed_data';
import 'package:http/http.dart' as http;
import 'api_client.dart';

class MediaUploadResult {
  final String id;
  final String url;
  final String contentType;
  final int sizeBytes;

  MediaUploadResult({
    required this.id,
    required this.url,
    required this.contentType,
    required this.sizeBytes,
  });

  factory MediaUploadResult.fromJson(Map<String, dynamic> json) {
    return MediaUploadResult(
      id: json['id'] as String,
      url: json['url'] as String,
      contentType: json['content_type'] as String,
      sizeBytes: json['size_bytes'] as int,
    );
  }
}

class MediaUploadService {
  final ApiClient apiClient;

  MediaUploadService({required this.apiClient});

  Future<MediaUploadResult> uploadFile({
    required Uint8List bytes,
    required String contentType,
    required String fileName,
  }) async {
    final uri = Uri.parse('${apiClient.baseUrl}/media/upload');
    final request = http.MultipartRequest('POST', uri);

    if (apiClient.accessToken != null) {
      request.headers['Authorization'] = 'Bearer ${apiClient.accessToken}';
    }

    request.files.add(
      http.MultipartFile.fromBytes(
        'file',
        bytes,
        filename: fileName,
      ),
    );

    final streamedResponse = await request.send();
    final response = await http.Response.fromStream(streamedResponse);

    if (response.statusCode >= 200 && response.statusCode < 300) {
      final json = jsonDecode(response.body);
      return MediaUploadResult.fromJson(json);
    } else {
      throw ApiException(response.statusCode, 'Failed to upload media');
    }
  }

  Future<String> getPresignedUrl(String mediaId) async {
    final uri = Uri.parse('${apiClient.baseUrl}/media/$mediaId/url');
    final response = await http.get(
      uri,
      headers: {
        if (apiClient.accessToken != null)
          'Authorization': 'Bearer ${apiClient.accessToken}',
      },
    );

    if (response.statusCode >= 200 && response.statusCode < 300) {
      final json = jsonDecode(response.body);
      return json['url'] as String;
    } else {
      throw ApiException(response.statusCode, 'Failed to get presigned URL');
    }
  }
}
