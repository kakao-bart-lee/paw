import Foundation

enum PawAccessibility {
    static let title = "paw.title"
    static let bridgeStatus = "paw.bridge.status"
    static let artifactsDirectory = "paw.bridge.artifacts"
    static let shellBanner = "paw.conversations.banner"
    static let currentAuthStep = "paw.auth.currentStep"
    static let authError = "paw.auth.error"
    static let pushStatus = "paw.push.status"
    static let connectionState = "paw.runtime.connectionState"
    static let composer = "paw.runtime.composer"
    static let messageList = "paw.chat.messages"
    static let conversationsEmpty = "paw.conversations.empty"
    static let chatEmpty = "paw.chat.empty"
    static let profileSummary = "paw.profile.summary"
    static let settingsSummary = "paw.settings.summary"
    static func authButton(_ step: PawAuthStep) -> String {
        "paw.auth.button.\(step.rawValue)"
    }
    static let sendMessageButton = "paw.chat.send"
    static let nextConversationButton = "paw.chat.nextConversation"
    static let activeLifecycleButton = "paw.lifecycle.active"
    static let backgroundLifecycleButton = "paw.lifecycle.background"
    static let registerPushButton = "paw.push.register"
    static let unregisterPushButton = "paw.push.unregister"
}
