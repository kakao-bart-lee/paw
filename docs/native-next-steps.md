# Native Next Steps

Updated: 2026-03-14

This document captures the recommended post-foundation work after merging `native-foundation`.

## Recommended Order

### 1. Native productization
Move Android/iOS from foundation shells to production-oriented app flows.

Focus:
- post-auth conversations/chat UX
- profile/settings completion
- error/loading/empty states
- push/runtime lifecycle polish

### 2. Visual QA pass
Run a dedicated screenshot + interaction pass for:
- Android
- iOS
- Web
- Desktop

Goal:
- confirm palette consistency
- confirm radius consistency
- confirm typography hierarchy
- confirm light/dark mode behavior where supported

### 3. Native theme mode support
Flutter Web/Desktop now has persisted theme-mode control.
Native shells can adopt equivalent user-facing theme switching if that remains in scope.

### 4. Technical debt cleanup
Candidates:
- repo-wide Flutter analyze warnings
- Android lint environment fragility on some hosts
- remaining UI-specific hardcoded colors/radii outside the latest polish scope

## Definition of Next Done

The next stream can be considered done when:
- Android/iOS feel like product surfaces instead of foundation shells
- visual QA screenshots are approved across all supported platforms
- no critical CI/review blockers remain
- release/handoff notes are updated
