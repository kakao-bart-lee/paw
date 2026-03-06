import 'package:flutter/material.dart';

enum ReadReceiptStatus { sent, delivered, read }

class ReadReceiptIndicator extends StatelessWidget {
  final ReadReceiptStatus status;

  const ReadReceiptIndicator({
    super.key,
    required this.status,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.check, size: 12, color: _color),
        if (status != ReadReceiptStatus.sent)
          Padding(
            padding: const EdgeInsets.only(left: 0),
            child: Icon(Icons.check, size: 12, color: _color),
          ),
      ],
    );
  }

  Color get _color => status == ReadReceiptStatus.read
      ? const Color(0xFF6C63FF)
      : Colors.grey;
}
