import Foundation

/// Thin facade that delegates to AuthViewModel, ChatViewModel, and BootstrapViewModel.
/// Preserved for backward compatibility with existing tests and views that reference PawCoreManager.
@MainActor
final class PawCoreManager: ObservableObject {
    @Published private(set) var bindingsStatus = "Native iOS bootstrap ready"
    @Published private(set) var preview: PawBootstrapPreview

    private let bootstrapVM: BootstrapViewModel

    init(
        tokenVault: PawTokenVault = PawKeychainTokenVault(),
        deviceKeyStore: PawDeviceKeyStore = PawKeychainDeviceKeyStore(),
        pushRegistrar: PawPushRegistrar = PawApnsPushRegistrar(),
        now: @escaping () -> Date = Date.init
    ) {
        let bootstrap = BootstrapViewModel(
            tokenVault: tokenVault,
            deviceKeyStore: deviceKeyStore,
            pushRegistrar: pushRegistrar,
            now: now
        )
        self.bootstrapVM = bootstrap
        self.preview = bootstrap.buildInitialPreview()
    }

    var artifactsDirectory: String {
        "PawCore/Artifacts"
    }

    var selectedConversation: PawConversationPreview? {
        bootstrapVM.selectedConversation(from: preview)
    }

    // MARK: - Auth delegation

    func startPhoneInput() {
        bootstrapVM.authVM.startPhoneInput(&preview)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func submitPhone(_ phone: String = "+82 10-5555-0101", discoverableByPhone: Bool = true) {
        bootstrapVM.authVM.submitPhone(phone, discoverableByPhone: discoverableByPhone, preview: &preview)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func verifyOtp(_ code: String = "123456") {
        bootstrapVM.authVM.verifyOtp(code, preview: &preview)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func submitDeviceName(_ deviceName: String = "Haruna's iPhone") {
        bootstrapVM.authVM.submitDeviceName(deviceName, preview: &preview)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func submitUsername(_ username: String = "haruna") {
        bootstrapVM.authVM.submitUsername(username, preview: &preview)
        let effectiveUsername = bootstrapVM.effectiveUsername(from: preview)
        bootstrapVM.chatVM.hydrateChatShell(
            authenticated: preview.auth.hasAccessToken,
            username: effectiveUsername,
            preview: &preview
        )
    }

    func skipUsername() {
        let effectiveUsername = bootstrapVM.effectiveUsername(from: preview)
        bootstrapVM.authVM.skipUsername(preview: &preview, effectiveUsername: effectiveUsername)
        bootstrapVM.chatVM.hydrateChatShell(
            authenticated: preview.auth.hasAccessToken,
            username: effectiveUsername,
            preview: &preview
        )
    }

    func logout() {
        bootstrapVM.authVM.logout(preview: &preview, pushRegistrar: bootstrapVM.pushRegistrar)
        bootstrapVM.chatVM.hydrateChatShell(
            authenticated: preview.auth.hasAccessToken,
            username: nil,
            preview: &preview
        )
    }

    // MARK: - Chat delegation

    func selectConversation(_ id: String) {
        let effectiveUsername = bootstrapVM.effectiveUsername(from: preview)
        bootstrapVM.chatVM.selectConversation(id, preview: &preview, effectiveUsername: effectiveUsername)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func selectNextConversation() {
        let effectiveUsername = bootstrapVM.effectiveUsername(from: preview)
        bootstrapVM.chatVM.selectNextConversation(preview: &preview, effectiveUsername: effectiveUsername)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func sendChatMessage(_ text: String? = nil) {
        let effectiveUsername = bootstrapVM.effectiveUsername(from: preview)
        bootstrapVM.chatVM.sendChatMessage(
            text,
            preview: &preview,
            effectiveUsername: effectiveUsername,
            agentReplyBuilder: { [preview, bootstrapVM] outgoing in
                bootstrapVM.agentReply(for: outgoing, preview: preview)
            }
        )
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    // MARK: - Bootstrap delegation

    func refresh() {
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func registerForPush(token: String = "apns-demo-token") {
        bootstrapVM.registerForPush(token: token, preview: &preview)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func unregisterPush() {
        bootstrapVM.unregisterPush(preview: &preview)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    func applyLifecycle(state: String) {
        bootstrapVM.applyLifecycle(state: state, preview: &preview)
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    nonisolated static func lifecycleHints(for state: String) -> [String] {
        BootstrapViewModel.lifecycleHints(for: state)
    }
}
