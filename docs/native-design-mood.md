# Paw Native Design Mood Guide

이 문서는 Flutter `AppTheme`의 시각적 분위기를 Android/iOS 네이티브 앱으로 옮길 때 기준으로 사용하는 디자인 메모입니다.

## 핵심 무드

- 거의 검정에 가까운 **청록 계열 다크 배경**
- `surface1~4` 로 나뉘는 **겹겹의 레이어**
- 얇은 outline과 큰 radius가 만드는 **부드러운 카드형 메신저 UI**
- 민트 그린 primary 강조
- Strong text / muted text 대비가 분명한 타이포
- AI 스트림, 보낸 메시지, 받은 메시지가 **톤으로 구분**됨

## Flutter 기준 토큰

Source: `paw-client/lib/core/theme/app_theme.dart`

- Background `#0B1113`
- Surface1 `#10181B`
- Surface2 `#141F23`
- Surface3 `#1A262B`
- Surface4 `#223137`
- Outline `#2A3C43`
- Primary `#63E6BE`
- Primary Soft `#15332C`
- Accent `#8EC5FF`
- Strong Text `#F5FAFC`
- Muted Text `#94A8AF`
- Sent Bubble `#1B7D66`
- Agent Bubble `#11262B`

## 적용 원칙

### Android
- `PawAndroidTheme`는 기본 라이트 테마가 아니라 Paw 다크 테마를 사용
- Compose card/message shell은 20~24dp radius 기준
- UI 스캐폴드 단계에서도 목록 카드/버블/bridge status 영역을 다른 surface tone으로 분리

### iOS
- `PawTheme.swift`, `PawTypography.swift`로 SwiftUI 쪽 토큰 고정
- 기본 `List` 흰 배경 대신 custom dark shell 사용
- 동일한 radius/outline/텍스트 대비를 유지

## 지금 반영된 위치

- Android: `paw-android/app/src/main/java/dev/paw/android/ui/theme/*`
- iOS: `paw-ios/Paw/PawTheme.swift`, `paw-ios/Paw/PawTypography.swift`

## 이후 네이티브 실제 화면 구현 시 유지할 것

- 대화 목록은 `surface2` 중심 카드형
- 보낸 메시지 = sentBubble
- 받은 메시지 = receivedBubble
- AI 스트림 = agentBubble
- 입력창은 `surface3` + outline 조합
- 배너류는 primarySoft / warning tint 기반으로 본문과 분리
