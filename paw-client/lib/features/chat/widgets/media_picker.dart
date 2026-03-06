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
            leading: const Icon(Icons.camera_alt),
            title: const Text('카메라'),
            trailing: Text(
              '준비 중',
              style: theme.textTheme.labelSmall?.copyWith(
                color: theme.colorScheme.error,
              ),
            ),
            onTap: null, // Disabled for Phase 2
          ),
          ListTile(
            leading: const Icon(Icons.image),
            title: const Text('갤러리'),
            onTap: () => _showStubMessage(context),
          ),
          ListTile(
            leading: const Icon(Icons.attach_file),
            title: const Text('파일'),
            onTap: () => _showStubMessage(context),
          ),
        ],
      ),
    );
  }
}
