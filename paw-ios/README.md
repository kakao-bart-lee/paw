# paw-ios

Minimal native iOS shell for Paw.

## Included scaffold

- `Paw.xcodeproj` with app + unit test targets
- `Paw/` SwiftUI app entrypoint and placeholder shell view
- `Paw/PawCoreManager.swift` bootstrap/auth preview store for future UniFFI integration
- `Paw/PawBootstrapView.swift` and `Paw/PawBootstrapModels.swift` for the native bootstrap/auth shell flow
- `PawCore/Artifacts/` reserved output path used by `scripts/build-core-ios.sh`

## Verify locally

```bash
cd paw-ios
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild -project Paw.xcodeproj -scheme Paw -destination 'platform=iOS Simulator,name=iPhone 17 Pro' build
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild -project Paw.xcodeproj -scheme Paw -destination 'platform=iOS Simulator,name=iPhone 17 Pro' test
```

## Current shell status

- The SwiftUI shell now mirrors the Android bootstrap/auth preview structure
- `xcodebuild ... build` succeeds with `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer`
- `xcodebuild ... test` succeeds against `iPhone 17 Pro`
- The next step is wiring generated UniFFI Swift bindings into this shell state
