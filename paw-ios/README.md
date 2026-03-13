# paw-ios

Minimal native iOS shell for Paw.

## Included scaffold

- `Paw.xcodeproj` with app + unit test targets
- `Paw/` SwiftUI app entrypoint and placeholder shell view
- `Paw/PawCoreManager.swift` placeholder bridge surface for future UniFFI integration
- `PawCore/Artifacts/` reserved output path used by `scripts/build-core-ios.sh`

## Verify locally

```bash
cd paw-ios
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild -project Paw.xcodeproj -scheme Paw -destination 'generic/platform=iOS Simulator' build
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild -project Paw.xcodeproj -scheme Paw -destination 'generic/platform=iOS Simulator' test
```

## Current build/tooling blockers

- The machine has `/Applications/Xcode.app`, but the active developer directory still points at Command Line Tools by default; set `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer` (or switch via `xcode-select`) before invoking `xcodebuild`.
- Even with full Xcode selected, the current environment is blocked on the iOS Simulator platform/runtime: `xcodebuild` reports that `iOS 26.2` is not installed under Xcode > Settings > Components.

## Blocker note

This worker environment only has `xcode-select` pointed at `/Library/Developer/CommandLineTools`, so `xcodebuild` cannot open/build the iOS project here. Install/select full Xcode before running the commands above.
