import 'package:flutter/material.dart';

class AgentScreen extends StatelessWidget {
  const AgentScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Agent')),
      body: const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.smart_toy_outlined, size: 64),
            SizedBox(height: 16),
            Text('Agent 마켓플레이스 (Phase 3에서 구현)'),
          ],
        ),
      ),
    );
  }
}
