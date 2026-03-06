# Paw Phase 1 Dogfooding Feedback Tracker

Track UX/product feedback from dogfooding sessions here.

## Status Legend

- `New`: newly reported, not triaged
- `Triaged`: reviewed and prioritized
- `Planned`: accepted and scheduled
- `In Progress`: actively being addressed
- `Done`: completed
- `Won't Do`: intentionally deferred or rejected

## Priority Legend

- `P0`: release blocker
- `P1`: high-impact usability/functional friction
- `P2`: moderate friction
- `P3`: low-impact polish

## Feedback Items

| ID | Date | Area | Feedback Item | Priority | Status | Owner | Notes |
|---|---|---|---|---|---|---|---|
| UX-001 | 2026-03-07 | Profile | `메시지 보내기` button is a stub and only shows SnackBar (`준비 중입니다`). | P1 | Triaged | Unassigned | Needs real navigation/start-chat flow in Phase 2. |
| UX-002 | 2026-03-07 | Media | `MediaPicker` bottom sheet appears but file picking/upload path is stubbed. | P1 | Triaged | Unassigned | Hook to native picker and upload pipeline. |
| UX-003 | 2026-03-07 | Local DB / Build | Drift `*.g.dart` files are stubs until `build_runner` is executed; creates onboarding confusion for testers. | P2 | Triaged | Unassigned | Add prebuilt artifacts or setup script for dogfooders. |
| UX-004 | 2026-03-07 | Setup | Flutter SDK unavailable in some environments blocks full client validation. | P2 | Triaged | Unassigned | Provide server-only checklist + CI-hosted client build guidance. |
| UX-005 | 2026-03-07 | Reliability Perception | Known `paw-crypto` compile issue (openmls) can be mistaken as Phase 1 breakage during setup. | P2 | Triaged | Unassigned | Explicitly mark out-of-scope in runbook and troubleshooting. |

## New Entry Template

Copy and append a row:

| UX-XXX | YYYY-MM-DD | Area | Feedback item | P1/P2/P3 | New | Owner | Notes |
