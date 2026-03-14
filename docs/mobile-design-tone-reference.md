# Paw Mobile Design Tone Reference

이 문서는 Paw iOS 앱을 이후에도 같은 디자인 톤으로 확장하기 위한 **UI/UX 기준 문서**입니다.

기준 소스:
- `/Users/haruna/Desktop/app_preview` 의 스크린샷 6장
- `paw-ios/Paw/Theme/PawTheme.swift`
- `paw-ios/Paw/Theme/PawTypography.swift`
- `paw-ios/Paw/Features/Auth/AuthView.swift`
- `paw-ios/Paw/Features/Chat/ChatView.swift`

---

## 1. 디자인의 목적

Paw의 모바일 UI는 일반적인 밝고 친근한 메신저가 아니라,

- **조용하고**
- **보안 중심이며**
- **AI와 사람이 함께 일하는 분위기**를 가지는
- **어두운 도구형 인터페이스**

로 유지되어야 합니다.

핵심 인상은 다음과 같습니다.

- secure messenger first
- AI-native collaboration second
- low-noise, high-focus
- premium but restrained

---

## 2. 핵심 톤 앤 매너

### 유지해야 하는 분위기
- 거의 검정에 가까운 배경
- 시끄럽지 않은 대비
- 여백이 많은 구성
- 장식보다 구조와 리듬 중심
- SF 느낌은 있지만 과하지 않음
- 기능이 많아져도 “조용한 앱”처럼 보여야 함

### 피해야 하는 방향
- 밝은 블루 위주의 SaaS 대시보드 느낌
- 라운드 카드가 많은 일반 메신저 느낌
- 과도한 글로우 / 네온 / 애니메이션
- 크고 무거운 CTA 버튼 남발
- bubble 중심의 전형적인 채팅 UI 남용

---

## 3. 컬러 시스템

기본 토큰은 `PawTheme.swift`를 기준으로 사용합니다.

### 기본 계열
- `background`: 앱 전체 바탕
- `backgroundGlow`: 중심부의 아주 약한 warm glow
- `surface1`, `surface2`, `surface3`: 단계별 표면
- `outline`: 구분선과 얇은 스트로크

### 텍스트
- `strongText`: 주요 텍스트
- `mutedText`: 보조 텍스트
- `subtleText`: 더 약한 메타 정보

### 의미 색상
- `teal`: AI / Signal / active state
- `amber`: security / identity / verified / encryption
- `lavender`: collective / alternative identity accent
- `coral`: human or thread accent
- `danger`: destructive action only

### 사용 규칙
- AI 관련 상태는 **teal**
- 보안 / 인증 / 신뢰 상태는 **amber**
- 파괴적 액션만 **danger**
- 컬러는 장식이 아니라 **의미 전달용**으로 사용

---

## 4. 타이포그래피 규칙

기본적으로 **monospaced typography**를 유지합니다.

### 사용 원칙
- 큰 제목도 과하게 굵지 않음
- 정보 구조는 굵기보다 **간격/대문자/위치**로 구분
- 섹션명, 탭, 메타 정보는 uppercase + tracking 사용
- body copy는 읽기 가능성을 해치지 않는 선에서 절제

### 대표 역할
- `hero`: 인증/브랜드 중심 장면
- `headlineLarge`, `headlineMedium`: 수치/아이덴티티/주요 타이틀
- `titleMedium`: 리스트 타이틀, 이름
- `bodyLarge`, `bodyMedium`, `bodySmall`: 본문 / 설명 / preview
- `labelMedium`, `labelSmall`: tab, meta, chips, time

---

## 5. 레이아웃 규칙

### 공통 구조
대부분의 화면은 아래 구조를 따릅니다.

1. status 영역
2. 최소 헤더
3. 주요 콘텐츠 영역
4. 하단 mode navigation 또는 입력 영역

### 레이아웃 원칙
- 화면 전체를 하나의 dark shell로 취급
- 카드 대신 **line + spacing + alignment**로 구조화
- 주요 콘텐츠는 왼쪽 기준선에 맞춰 정렬
- 세로 여백을 넉넉하게 사용
- 하단 navigation은 항상 조용하게 유지

### spacing 감각
- 바깥 여백: 20~30pt 수준
- row vertical padding: 16~18pt
- section 간 거리: 넓게
- divider는 얇고 희미하게

---

## 6. 핵심 화면 구조

## 6.1 AUTH

### Phone entry
- 브랜드 `Paw`
- 보조 문구 `SIGNAL YOUR PRESENCE`
- 중앙 정렬된 단일 입력
- 아주 절제된 primary action (`TRANSMIT`)

### OTP entry
- `SPEAK THE CODE`
- 숫자 슬롯 강조
- active underline
- 최소 chrome

### 확장 단계
- device name
- username

이 단계들도 동일하게:
- 한 화면 = 한 작업
- 입력 1개 중심
- 불필요한 설명 최소화
- 중심 정렬 유지

---

## 6.2 STREAM

### 필수 요소
- 상단 `STREAM` 레이블
- utility action (search / compose / plus)
- thread list
- 하단 `STREAM / SIGNALS / SELF`

### thread row 구성
- 왼쪽 vertical rail
- waveform signature
- title + time
- 필요 시 `SIGNAL` 또는 `COLLECTIVE` 메타
- 1줄 preview
- 오른쪽 status dot

### 표현 규칙
- 리스트는 카드 스택이 아니라 **divider 기반**
- row 하나하나가 얇게 새겨진 느낌이어야 함

---

## 6.3 SIGNALS

### 필수 요소
- `SIGNALS` 헤더
- 설명용 한 줄 subtitle
- signal 목록
- bind / unbind 상태
- ability chips

### signal row 구성
- left rail
- signal 이름
- domain label
- italic essence
- outlined ability chips
- 우측 상태 원형 인디케이터

### 상호작용 규칙
- tap 시 detail / permission overlay
- bind 상태는 checkmark + subtle ring으로 표현
- AI 관련 강조는 teal 유지

---

## 6.4 SELF

### 필수 요소
- `SELF` 헤더
- profile orb
- user identity / masked phone
- stats
- configuration list

### configuration row 구성
- 아이콘
- 설정명
- 현재 값
- 상태 dot

### 표현 규칙
- 프로필은 앱의 중심 identity anchor
- orb는 SELF에서만 특별하게 사용
- 설정은 시스템 도구처럼 보이게 유지

---

## 6.5 PRESENCE / CHAT DETAIL

### 필수 요소
- back
- title
- encrypted indicator
- status dot
- text-flow conversation
- composer

### 메시지 규칙
- 내 메시지: right aligned
- 상대 메시지: left aligned
- agent signal: centered + teal + italic
- bubble보다 **text block + separator line** 중심

### composer 규칙
- 하단 고정
- placeholder는 약하게
- SEND는 버튼처럼 크지 않게

---

## 7. 컴포넌트 규칙

### Header
- uppercase label + wide tracking
- utility action은 작고 가벼워야 함

### Divider
- 대부분의 구조는 divider가 담당
- 너무 진하게 쓰지 않음

### Chips / tags
- outlined only
- small uppercase mono
- filled pill은 특별한 경우만

### Status dots
- 상태 표현의 핵심
- 색만으로 의미 전달하지 말고 주변 텍스트와 함께 사용

### Overlay / modal
- 전체 dim layer 위에 얇은 border shell
- 무거운 카드 느낌보다 “layer” 느낌
- 정보 + action을 단순하게 유지

---

## 8. 새 기능 추가 시 규칙

## 8.1 검색
- STREAM의 확장으로 보이게 해야 함
- 별도 완전히 다른 화면보다 overlay 우선
- 결과는 기존 thread row 재사용

## 8.2 새 대화 / 새 그룹
- “compose / weave” 개념으로 연결
- 일반 메신저형 floating composer보다
  Paw의 quiet tone 유지
- direct / collective 같은 semantic mode 사용 가능

## 8.3 Agent 상세 / 권한
- SIGNAL overlay로 표현
- permission bullets
- bind / unbind action
- teal semantics 유지

## 8.4 설정 상세
- SELF에서 drill-in
- value + toggle + 짧은 설명
- settings sheet도 dark shell 원칙 유지

## 8.5 보안 기능
- amber 기반
- verified / encrypted / end-to-end 같은 calm trust cue 사용
- 공포감 조성보다 신뢰감이 중요

---

## 9. 구현 원칙

새 UI를 구현할 때는 아래 순서를 지킵니다.

1. 먼저 기존 shell 안에서 해결 가능한지 본다.
2. 새 색을 만들기보다 기존 semantic token을 재사용한다.
3. 새 카드를 만들기보다 divider/list/overlay 패턴으로 해결한다.
4. 일반 메신저 관성보다 Paw의 tone을 우선한다.
5. 기능이 늘어나도 UI noise는 늘리지 않는다.

---

## 10. 개발 체크리스트

새 기능을 만들 때 아래를 확인합니다.

- [ ] background / outline / text 톤이 기존과 맞는가
- [ ] mono typography가 유지되는가
- [ ] AI는 teal, security는 amber로 일관적인가
- [ ] row / overlay / bottom nav 패턴을 재사용했는가
- [ ] 카드형 일반 메신저 UI로 흐르지 않았는가
- [ ] 기능은 늘었지만 조용한 느낌은 유지되는가

---

## 11. 현재 확장된 기능 범위

이 문서 작성 시점 기준 현재 screenshot-tone 레이아웃 위에 다음 기능이 반영되었습니다.

- auth flow tone remap
- stream thread list
- presence conversation detail
- signal bind / unbind overlay
- stream search overlay
- new thread / collective compose overlay
- self settings detail overlay
- dev mode direct entry into STREAM

이후 기능을 추가할 때도 위 구조를 우선 유지합니다.
