import 'package:flutter/material.dart';

class AvatarWidget extends StatelessWidget {
  final String? imageUrl;
  final String displayName;
  final double size;

  const AvatarWidget({
    super.key,
    this.imageUrl,
    required this.displayName,
    this.size = 48,
  });

  Color _colorFromName(String name) {
    const colors = [
      Color(0xFF42A5F5), // blue 400
      Color(0xFF66BB6A), // green 400
      Color(0xFFFFA726), // orange 400
      Color(0xFFAB47BC), // purple 400
      Color(0xFF26A69A), // teal 400
      Color(0xFFEF5350), // red 400
    ];
    if (name.isEmpty) return colors[0];
    final sum = name.codeUnits.fold(0, (a, b) => a + b);
    return colors[sum % colors.length];
  }

  @override
  Widget build(BuildContext context) {
    final radius = size / 2;
    if (imageUrl != null && imageUrl!.isNotEmpty) {
      return CircleAvatar(
        radius: radius,
        backgroundImage: NetworkImage(imageUrl!),
      );
    }
    final initial =
        displayName.isNotEmpty ? displayName[0].toUpperCase() : '?';
    return CircleAvatar(
      radius: radius,
      backgroundColor: _colorFromName(displayName),
      child: Text(
        initial,
        style: TextStyle(
          fontSize: radius * 0.8,
          color: Colors.white,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }
}
