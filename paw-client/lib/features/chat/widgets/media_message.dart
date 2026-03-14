import 'package:flutter/material.dart';
import '../../../core/theme/app_theme.dart';

class MediaMessage extends StatelessWidget {
  final String mediaId;
  final String contentType;
  final String? fileName;
  final int? sizeBytes;
  final bool isMe;

  const MediaMessage({
    super.key,
    required this.mediaId,
    required this.contentType,
    this.fileName,
    this.sizeBytes,
    required this.isMe,
  });

  bool get isImage => contentType.startsWith('image/');

  String _formatSize(int? bytes) {
    if (bytes == null) return 'Unknown size';
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final textColor = isMe ? AppTheme.background : theme.colorScheme.onSurface;
    final iconColor = isMe
        ? AppTheme.background.withValues(alpha: 0.78)
        : theme.colorScheme.onSurfaceVariant;

    if (isImage) {
      return GestureDetector(
        onTap: () {
          // Phase 2: Open full-screen image
        },
        child: ClipRRect(
          borderRadius: BorderRadius.circular(12),
          child: Container(
            width: 240,
            height: 240,
            color: AppTheme.surface3,
            child: const Center(
              child: Icon(Icons.image, size: 48, color: Colors.grey),
              // Phase 2: Load actual image using presigned URL
            ),
          ),
        ),
      );
    }

    // File message
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: isMe
                ? AppTheme.background.withValues(alpha: 0.2)
                : AppTheme.surface3,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Icon(
            Icons.insert_drive_file,
            color: isMe ? AppTheme.background : AppTheme.accent,
          ),
        ),
        const SizedBox(width: 12),
        Flexible(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                fileName ?? 'Unknown file',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: textColor,
                  fontWeight: FontWeight.bold,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              const SizedBox(height: 4),
              Text(
                _formatSize(sizeBytes),
                style: theme.textTheme.labelSmall?.copyWith(
                  color: iconColor,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
