import Foundation

/// Coordinates AuthViewModel + ChatViewModel, manages session restore,
/// lifecycle, push registration, and the composite preview state.
@MainActor
final class BootstrapViewModel {
    private(set) var authVM: AuthViewModel
    private(set) var chatVM: ChatViewModel

    private let tokenVault: PawTokenVault
    private let deviceKeyStore: PawDeviceKeyStore
    let pushRegistrar: PawPushRegistrar

    init(
        tokenVault: PawTokenVault,
        deviceKeyStore: PawDeviceKeyStore,
        pushRegistrar: PawPushRegistrar,
        now: @escaping () -> Date = Date.init
    ) {
        self.tokenVault = tokenVault
        self.deviceKeyStore = deviceKeyStore
        self.pushRegistrar = pushRegistrar
        self.authVM = AuthViewModel(tokenVault: tokenVault, deviceKeyStore: deviceKeyStore)
        self.chatVM = ChatViewModel(now: now)
    }

    func buildInitialPreview() -> PawBootstrapPreview {
        let hasDeviceKey = deviceKeyStore.hasDeviceKey()
        let tokens = tokenVault.loadTokens()
        let shell = ChatViewModel.makeShellState(
            authenticated: tokens.accessToken != nil,
            username: nil
        )
        return PawBootstrapPreview(
            bridgeStatus: "Keychain + APNs adapters wired",
            auth: AuthViewModel.makeAuthPreview(tokens: tokens),
            runtime: PawRuntimePreview(
                connectionState: tokens.accessToken == nil ? "Disconnected" : "Ready",
                cursorCount: shell.conversations.count,
                activeStreamCount: 0
            ),
            storage: tokenVault.storagePreview(hasDeviceKey: hasDeviceKey),
            push: pushRegistrar.currentState(),
            lifecycle: PawLifecyclePreview(
                activeHints: Self.lifecycleHints(for: "Active"),
                backgroundHints: Self.lifecycleHints(for: "Background"),
                currentState: "Launching"
            ),
            conversations: shell.conversations,
            selectedConversationID: shell.selectedConversationID,
            messages: shell.messages,
            composerText: shell.composerText,
            shellBanner: shell.banner
        )
    }

    func registerForPush(token: String = "apns-demo-token", preview: inout PawBootstrapPreview) {
        preview.push = pushRegistrar.register(token: token)
    }

    func unregisterPush(preview: inout PawBootstrapPreview) {
        preview.push = pushRegistrar.unregister()
    }

    func applyLifecycle(state: String, preview: inout PawBootstrapPreview) {
        preview.lifecycle.currentState = state
        switch state {
        case "Active":
            preview.lifecycle.activeHints = Self.lifecycleHints(for: state)
            preview.runtime.connectionState = preview.auth.hasAccessToken ? "Connected" : "Disconnected"
        case "Background":
            preview.lifecycle.backgroundHints = Self.lifecycleHints(for: state)
            preview.runtime.connectionState = "Background"
            preview.runtime.activeStreamCount = 0
        default:
            break
        }
    }

    func updateShellBanner(preview: inout PawBootstrapPreview) {
        if !preview.auth.hasAccessToken {
            preview.shellBanner = "Authenticate to unlock conversations + chat runtime shell."
            return
        }

        let selectedConversation = selectedConversation(from: preview)
        let conversation = selectedConversation?.title ?? "No conversation"
        preview.shellBanner = "\(conversation) · \(preview.runtime.connectionState) · push \(preview.push.status.lowercased())"
    }

    func agentReply(for outgoing: String, preview: PawBootstrapPreview) -> String {
        let pushState = preview.push.status == "Registered" ? "APNs active" : "APNs pending"
        let lifecycle = preview.lifecycle.currentState
        return "Runtime live: \(preview.runtime.connectionState), \(pushState), lifecycle=\(lifecycle). Echoing \"\(outgoing)\" into the iOS shell."
    }

    func selectedConversation(from preview: PawBootstrapPreview) -> PawConversationPreview? {
        guard let id = preview.selectedConversationID else { return nil }
        return preview.conversations.first(where: { $0.id == id })
    }

    func effectiveUsername(from preview: PawBootstrapPreview) -> String {
        preview.auth.username.isEmpty ? "haruna" : preview.auth.username
    }

    nonisolated static func lifecycleHints(for state: String) -> [String] {
        switch state {
        case "Launching", "Active":
            ["ReconnectSocket", "RefreshPushToken"]
        case "Inactive":
            ["FlushAcks"]
        case "Background", "Terminated":
            ["PauseRealtime", "FlushAcks", "PersistDrafts"]
        default:
            []
        }
    }
}
