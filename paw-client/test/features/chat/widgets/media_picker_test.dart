import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:paw_client/features/chat/widgets/media_picker.dart';

void main() {
  testWidgets('shows web/desktop upload actions instead of mobile camera flow', (
    tester,
  ) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: MediaPicker(onFilePicked: _noop),
        ),
      ),
    );

    expect(find.text('이미지 파일'), findsOneWidget);
    expect(find.text('파일 업로드'), findsOneWidget);
    expect(find.text('카메라'), findsNothing);
    expect(find.text('갤러리'), findsNothing);
  });
}

void _noop(String unusedPath, String unusedContentType) {}
