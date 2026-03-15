여기서 도출되는 ‘차세대 메신저 플랫폼/서비스’의 필수 기능
	1.	네이티브 AI 스트리밍/드래프트 전송 계층
메신저는 partial output을 edit spam 없이 흘릴 수 있는 draft/stream primitive를 기본 제공해야 합니다. Telegram의 sendMessageDraft와 “every bot can enable streaming responses”는 이 방향을 보여주고, 반대로 Matrix 채널의 intermediate output 노출 이슈는 그런 제어가 없을 때 UX가 얼마나 쉽게 망가지는지를 보여줍니다.  ￼
	2.	thread/topic/room 단위의 문맥 격리
AI assistant 시대의 메신저는 단순 message list가 아니라, thread/topic별로 독립 세션을 가질 수 있어야 합니다. OpenClaw는 Telegram forum topic을 이미 개별 session key로 처리하고, Matrix는 그 부재 때문에 per-thread isolation 요청이 올라왔습니다. 이건 미래형 메신저의 핵심 설계 요소입니다.  ￼
	3.	텍스트가 아니라 ‘구조화된 메시지 객체’
poll, reaction, location, date_time, voice/video note, role tag 같은 객체가 네이티브여야 합니다. Telegram은 최근 date/time entity, member tags, poll timestamps, bot streaming을 넓혔고, Matrix도 polls·location·reactions·rooms/threads를 지원합니다. 차세대 메신저는 텍스트 위에 AI를 얹는 수준이 아니라, 구조화된 stateful objects를 AI가 조작할 수 있어야 합니다.  ￼
	4.	세밀한 신뢰·권한 모델
DM pairing, allowlist, mention gating, per-room access, per-action gating 같은 안전장치가 기본이어야 합니다. OpenClaw는 이미 unknown sender에 pairing code를 주고, group reply를 allowlist와 mention gating으로 제한합니다. 차세대 메신저는 이런 모델을 플랫폼 차원에서 지원해야 prompt-injection과 오작동을 줄일 수 있습니다.  ￼
	5.	프라이버시와 검증 가능 보안
E2EE, device verification, encrypted media, disable-sharing 같은 기능이 “고급 옵션”이 아니라 기본값이어야 합니다. Matrix는 OpenClaw 수준에서도 device verification과 encrypted media를 다루고 있고, Telegram은 최근 1:1 chat의 disable sharing을 넣었습니다. 앞으로 AI assistant가 대화 안에 깊게 들어올수록 이 축은 더 중요해집니다.  ￼
	6.	오픈성·이식성·멀티호밍
한 vendor의 API 변경 하나로 핵심 기능이 깨지지 않아야 합니다. Matrix의 open standard/any homeserver/multi-account 방향은 여기에 가깝고, Slack의 deprecated files.upload 사례는 반대로 폐쇄 API 의존이 얼마나 취약한지 보여줍니다. 차세대 메신저는 federation이든, self-host든, 최소한 안정된 export/import/bridge와 versioned developer platform을 제공해야 합니다.  ￼
	7.	멀티모달·실시간 협업
음성/영상/위치/파일/이모지/실시간 방 구조가 AI assistant와 한 화면 안에서 이어져야 합니다. Matrix 최근 업데이트의 live location sharing, voice/video rooms, Spaces 진화는 이 방향을 보여줍니다. Telegram도 voice/video note, polls, member tags 쪽이 빠르게 넓어지고 있습니다.  ￼
	8.	운영 신뢰성과 관측 가능성
AI 시대 메신저는 “보내기는 되는데 받기는 안 되는” 식의 반쪽 상태를 허용하면 안 됩니다. 최근 Telegram의 silent inbound polling failure, WhatsApp의 linked/OK인데 inbound 미수신, Slack의 session regression은 메신저 플랫폼이 health semantics와 observability를 얼마나 명확히 제공해야 하는지를 보여줍니다.  ￼
