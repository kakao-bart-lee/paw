Konnect 프론트엔드의 컬러 시스템 문서입니다.
`frontend/src/index.css`의 CSS 변수 기반 Tailwind 테마에서 추출했습니다.

> **팔레트 컨셉**: "arscontexta-inspired warm palette"
> Light: warm parchment 계열 (크림/베이지 배경 + 짙은 갈색 텍스트)
> Dark: olive-black 계열 (녹색끼 있는 매우 어두운 배경)
> Accent: Amber gold (`#c8832a`) — light/dark 모두 동일 유지

---

## Light Mode

| 토큰 | HSL | Hex | 용도 |
|------|-----|-----|------|
| `--background` | `38 33% 95%` | `#f5f0e8` | 페이지 배경 |
| `--foreground` | `36 8% 16%` | `#2c2a25` | 기본 텍스트 |
| `--card` | `39 30% 97%` | `#f8f5f0` | 카드 배경 |
| `--card-foreground` | `36 8% 16%` | `#2c2a25` | 카드 텍스트 |
| `--popover` | `39 30% 97%` | `#f8f5f0` | 팝오버 배경 |
| `--popover-foreground` | `36 8% 16%` | `#2c2a25` | 팝오버 텍스트 |
| `--primary` | `36 8% 16%` | `#2c2a25` | 기본 버튼/강조 배경 |
| `--primary-foreground` | `38 33% 95%` | `#f5f0e8` | primary 위 텍스트 |
| `--secondary` | `37 18% 90%` | `#ebe5d9` | 보조 버튼 배경 |
| `--secondary-foreground` | `36 8% 20%` | `#363330` | 보조 버튼 텍스트 |
| `--muted` | `37 18% 90%` | `#ebe5d9` | 흐린 배경 |
| `--muted-foreground` | `30 6% 46%` | `#7a7570` | 흐린 텍스트 (placeholder 등) |
| `--accent` | `35 60% 52%` | `#c8832a` | 강조색 (amber gold) |
| `--accent-foreground` | `36 8% 16%` | `#2c2a25` | accent 위 텍스트 |
| `--destructive` | `0 72% 51%` | `#e03030` | 삭제/오류 |
| `--destructive-foreground` | `38 33% 95%` | `#f5f0e8` | destructive 위 텍스트 |
| `--border` | `36 16% 84%` | `#ddd7cc` | 테두리 |
| `--input` | `36 16% 84%` | `#ddd7cc` | 입력 필드 테두리 |
| `--ring` | `35 60% 52%` | `#c8832a` | 포커스 링 |
| `--radius` | — | `0.375rem` | 기본 border-radius |

### Light Mode — Sidebar

| 토큰 | HSL | Hex | 용도 |
|------|-----|-----|------|
| `--sidebar-background` | `38 25% 93%` | `#f0ece3` | 사이드바 배경 |
| `--sidebar-foreground` | `36 8% 22%` | `#3a3730` | 사이드바 기본 텍스트 |
| `--sidebar-primary` | `36 8% 16%` | `#2c2a25` | 사이드바 active 항목 배경 |
| `--sidebar-primary-foreground` | `38 33% 95%` | `#f5f0e8` | 사이드바 active 항목 텍스트 |
| `--sidebar-accent` | `37 18% 88%` | `#e8e2d6` | 사이드바 hover 배경 |
| `--sidebar-accent-foreground` | `36 8% 16%` | `#2c2a25` | 사이드바 hover 텍스트 |
| `--sidebar-border` | `36 12% 86%` | `#e0dbd2` | 사이드바 구분선 |
| `--sidebar-ring` | `35 60% 52%` | `#c8832a` | 사이드바 포커스 링 |

---

## Dark Mode

`.dark` 클래스 또는 `prefers-color-scheme: dark` 미디어 쿼리 적용.

| 토큰 | HSL | Hex | 용도 |
|------|-----|-----|------|
| `--background` | `70 8% 5%` | `#0c0d0b` | 페이지 배경 |
| `--foreground` | `38 33% 93%` | `#f0ebe0` | 기본 텍스트 |
| `--card` | `70 6% 7.5%` | `#131412` | 카드 배경 |
| `--card-foreground` | `38 33% 93%` | `#f0ebe0` | 카드 텍스트 |
| `--popover` | `70 6% 7.5%` | `#131412` | 팝오버 배경 |
| `--popover-foreground` | `38 33% 93%` | `#f0ebe0` | 팝오버 텍스트 |
| `--primary` | `38 33% 93%` | `#f0ebe0` | 기본 버튼/강조 배경 |
| `--primary-foreground` | `70 8% 5%` | `#0c0d0b` | primary 위 텍스트 |
| `--secondary` | `60 4% 12%` | `#1f1f1e` | 보조 버튼 배경 |
| `--secondary-foreground` | `38 28% 88%` | `#e5dfd4` | 보조 버튼 텍스트 |
| `--muted` | `60 4% 12%` | `#1f1f1e` | 흐린 배경 |
| `--muted-foreground` | `36 10% 55%` | `#9a9086` | 흐린 텍스트 |
| `--accent` | `35 60% 52%` | `#c8832a` | 강조색 (light와 동일) |
| `--accent-foreground` | `38 33% 93%` | `#f0ebe0` | accent 위 텍스트 |
| `--destructive` | `0 62% 35%` | `#902020` | 삭제/오류 (어둡게 조정) |
| `--destructive-foreground` | `38 33% 93%` | `#f0ebe0` | destructive 위 텍스트 |
| `--border` | `50 5% 15%` | `#262624` | 테두리 |
| `--input` | `50 5% 15%` | `#262624` | 입력 필드 테두리 |
| `--ring` | `35 60% 52%` | `#c8832a` | 포커스 링 |

### Dark Mode — Sidebar

| 토큰 | HSL | Hex | 용도 |
|------|-----|-----|------|
| `--sidebar-background` | `70 8% 4%` | `#0a0b09` | 사이드바 배경 (배경보다 더 어둡게) |
| `--sidebar-foreground` | `38 28% 88%` | `#e5dfd4` | 사이드바 기본 텍스트 |
| `--sidebar-primary` | `35 60% 52%` | `#c8832a` | 사이드바 active 항목 (amber) |
| `--sidebar-primary-foreground` | `70 8% 5%` | `#0c0d0b` | 사이드바 active 항목 텍스트 |
| `--sidebar-accent` | `60 4% 10%` | `#1a1a19` | 사이드바 hover 배경 |
| `--sidebar-accent-foreground` | `38 28% 88%` | `#e5dfd4` | 사이드바 hover 텍스트 |
| `--sidebar-border` | `50 5% 12%` | `#1f1f1d` | 사이드바 구분선 |
| `--sidebar-ring` | `35 60% 52%` | `#c8832a` | 사이드바 포커스 링 |

---

## Chart 컬러

데이터 시각화(차트)에 사용되는 컬러입니다.

| 토큰 | Light Hex | Dark Hex | 비고 |
|------|-----------|----------|------|
| `--chart-1` | `#c8832a` | `#c8832a` | Amber — 동일 유지 |
| `--chart-2` | `#3d9972` | `#4ab87f` | Teal (dark에서 밝게) |
| `--chart-3` | `#3d6070` | `#d49240` | Slate → Warm Orange |
| `--chart-4` | `#c08038` | `#8855b8` | Orange → Purple |
| `--chart-5` | `#b84030` | `#c8355a` | Red-orange → Crimson |

---

## 주요 색상 스와치

```
Light Mode
┌─────────────────────────────────────────────────────┐
│ Background  #f5f0e8  ░░░  Foreground  #2c2a25  ██   │
│ Card        #f8f5f0  ░░░  Border      #ddd7cc  ▒▒▒  │
│ Muted       #ebe5d9  ░░░  Muted FG    #7a7570  ▒▒   │
│ Accent      #c8832a  ▓▓▓  Destructive #e03030  ███  │
│ Sidebar BG  #f0ece3  ░░░  Sidebar Br  #e0dbd2  ▒▒   │
└─────────────────────────────────────────────────────┘

Dark Mode
┌─────────────────────────────────────────────────────┐
│ Background  #0c0d0b  ██   Foreground  #f0ebe0  ░░░  │
│ Card        #131412  ██   Border      #262624  ▒▒   │
│ Muted       #1f1f1e  ██   Muted FG    #9a9086  ▒▒▒  │
│ Accent      #c8832a  ▓▓▓  Destructive #902020  ▓▓   │
│ Sidebar BG  #0a0b09  ██   Sidebar Ac  #c8832a  ▓▓▓  │
└─────────────────────────────────────────────────────┘
```
