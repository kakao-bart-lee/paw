import Foundation

struct PawBootstrapPreview {
    var bridgeStatus: String
    var auth: PawAuthPreview
    var runtime: PawRuntimePreview
    var storage: PawStoragePreview
    var push: PawPushPreview
    var lifecycle: PawLifecyclePreview
}

struct PawAuthPreview {
    var step: String
    var discoverableByPhone: Bool
    var hasAccessToken: Bool
}

struct PawRuntimePreview {
    var connectionState: String
    var cursorCount: Int
    var activeStreamCount: Int
}

struct PawStoragePreview {
    var provider: String
    var availability: String
}

struct PawPushPreview {
    var status: String
    var platform: String
}

struct PawLifecyclePreview {
    var activeHints: [String]
    var backgroundHints: [String]
}
