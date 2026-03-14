import Foundation

struct PawBootstrapPreview {
    var bridgeStatus: String
    var auth: PawAuthPreview
    var runtime: PawRuntimePreview
    var storage: PawStoragePreview
    var push: PawPushPreview
    var lifecycle: PawLifecyclePreview
    var conversations: [PawConversationPreview]
    var selectedConversationID: String?
    var messages: [PawMessagePreview]
    var composerText: String
    var shellBanner: String
}

struct PawRuntimePreview {
    var connectionState: String
    var cursorCount: Int
    var activeStreamCount: Int
}

struct PawStoragePreview {
    var provider: String
    var availability: String
    var hasDeviceKey: Bool
}

struct PawPushPreview {
    var status: String
    var platform: String
    var token: String?
    var lastError: String?
    var lastUpdatedMs: Int
}

struct PawLifecyclePreview {
    var activeHints: [String]
    var backgroundHints: [String]
    var currentState: String
}
