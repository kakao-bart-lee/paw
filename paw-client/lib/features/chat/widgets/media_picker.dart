import 'package:flutter/material.dart';

class MediaPicker extends StatelessWidget {
  final Function(String filePath, String contentType) onFilePicked;

  const MediaPicker({super.key, required this.onFilePicked});

  static Future<void> show(
    BuildContext context, {
    required Function(String, String) onFilePicked,
  }) {
    return showModalBottomSheet(
      context: context,
      builder: (_) => MediaPicker(onFilePicked: onFilePicked),
    );
  }

  void _showStubMessage(BuildContext context) {
    Navigator.pop(context);
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('미디어 선택 기능은 곧 추가됩니다')),
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          ListTile(
            leading: const Icon(Icons.image_outlined),
            title: const Text('이미지 파일'),
            subtitle: const Text('Web/Desktop에서 이미지를 업로드합니다'),
            onTap: () => _showStubMessage(context),
          ),
          ListTile(
            leading: const Icon(Icons.attach_file),
            title: const Text('파일 업로드'),
            subtitle: const Text('문서 또는 일반 파일을 선택합니다'),
            onTap: () => _showStubMessage(context),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
            child: Text(
              '카메라/갤러리 기반 모바일 첨부 흐름은 네이티브 앱으로 이전됩니다.',
              style: theme.textTheme.bodySmall,
            ),
          ),
        ],
      ),
    );
  }
}
