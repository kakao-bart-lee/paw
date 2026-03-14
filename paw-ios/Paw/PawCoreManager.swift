import Foundation
#if os(iOS)
import UIKit
#endif

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

    var isDevelopmentAuthBypassEnabled: Bool {
#if DEBUG
        ProcessInfo.processInfo.environment["PAW_UI_TEST_MODE"] != "1"
#else
        false
#endif
    }

    // MARK: - Auth delegation (synchronous, backward-compat)

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

    /// Dev-only: bypass interactive auth and go straight to AUTHENTICATED.
    func devQuickLogin(
        phone: String? = nil,
        deviceName: String? = nil,
        username: String? = nil,
        discoverableByPhone: Bool = true
    ) {
        guard isDevelopmentAuthBypassEnabled else {
            startPhoneInput()
            return
        }

        let resolvedPhone = normalized(phone) ?? "+82 10-0000-0000"
        let resolvedDeviceName = normalized(deviceName) ?? Self.defaultDevelopmentDeviceName
        let resolvedUsername = normalized(username) ?? "dev"

        bootstrapVM.authVM.submitPhone(
            resolvedPhone,
            discoverableByPhone: discoverableByPhone,
            preview: &preview
        )
        bootstrapVM.authVM.verifyOtp("000000", preview: &preview)
        bootstrapVM.authVM.submitDeviceName(resolvedDeviceName, preview: &preview)
        bootstrapVM.authVM.submitUsername(resolvedUsername, preview: &preview)

        preview.auth.phone = resolvedPhone
        preview.auth.deviceName = resolvedDeviceName
        preview.auth.username = resolvedUsername
        preview.auth.discoverableByPhone = discoverableByPhone
        preview.auth.hasSessionToken = true
        preview.auth.hasAccessToken = true
        preview.auth.hasRefreshToken = true
        preview.auth.isLoading = false
        preview.auth.error = nil
        preview.auth.step = .authenticated
        preview.runtime.connectionState = "Connected"

        bootstrapVM.chatVM.hydrateChatShell(
            authenticated: true,
            username: resolvedUsername,
            preview: &preview
        )
        bootstrapVM.updateShellBanner(preview: &preview)
    }

    // MARK: - Auth delegation (async, API-backed, used by views)

    func submitPhoneAsync(_ phone: String) {
        let vm = bootstrapVM.authVM
        let bvm = bootstrapVM
        Task {
            var p = self.preview
            await vm.submitPhoneAsync(phone, preview: &p)
            bvm.updateShellBanner(preview: &p)
            self.preview = p
        }
    }

    func verifyOtpAsync(_ code: String) {
        let vm = bootstrapVM.authVM
        let bvm = bootstrapVM
        Task {
            var p = self.preview
            await vm.verifyOtpAsync(code, preview: &p)
            bvm.updateShellBanner(preview: &p)
            self.preview = p
        }
    }

    func submitDeviceNameAsync(_ deviceName: String) {
        let vm = bootstrapVM.authVM
        let bvm = bootstrapVM
        Task {
            var p = self.preview
            await vm.submitDeviceNameAsync(deviceName, preview: &p)
            bvm.updateShellBanner(preview: &p)
            self.preview = p
        }
    }

    func submitUsernameAsync(_ username: String, discoverableByPhone: Bool) {
        let authVM = bootstrapVM.authVM
        let chatVM = bootstrapVM.chatVM
        let bvm = bootstrapVM
        Task {
            var p = self.preview
            await authVM.submitUsernameAsync(
                username,
                discoverableByPhone: discoverableByPhone,
                preview: &p
            )
            let effectiveUsername = bvm.effectiveUsername(from: p)
            await chatVM.hydrateChatFromAPI(
                authenticated: p.auth.hasAccessToken,
                username: effectiveUsername,
                preview: &p
            )
            self.preview = p
        }
    }

    func skipUsernameAsync() {
        let bvm = bootstrapVM
        let chatVM = bootstrapVM.chatVM
        let effectiveUsername = bvm.effectiveUsername(from: preview)
        bvm.authVM.skipUsername(preview: &preview, effectiveUsername: effectiveUsername)
        Task {
            var p = self.preview
            await chatVM.hydrateChatFromAPI(
                authenticated: p.auth.hasAccessToken,
                username: effectiveUsername,
                preview: &p
            )
            self.preview = p
        }
    }

    // MARK: - Chat delegation (synchronous, backward-compat)

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

    // MARK: - Chat delegation (async, API-backed, used by views)

    func selectConversationAsync(_ id: String) {
        let chatVM = bootstrapVM.chatVM
        let bvm = bootstrapVM
        Task {
            var p = self.preview
            let effectiveUsername = bvm.effectiveUsername(from: p)
            await chatVM.selectConversationAsync(
                id,
                preview: &p,
                effectiveUsername: effectiveUsername
            )
            bvm.updateShellBanner(preview: &p)
            self.preview = p
        }
    }

    func selectNextConversationAsync() {
        let chatVM = bootstrapVM.chatVM
        let bvm = bootstrapVM
        Task {
            var p = self.preview
            let effectiveUsername = bvm.effectiveUsername(from: p)
            await chatVM.selectNextConversationAsync(
                preview: &p,
                effectiveUsername: effectiveUsername
            )
            bvm.updateShellBanner(preview: &p)
            self.preview = p
        }
    }

    func sendChatMessageAsync(_ text: String? = nil) {
        let chatVM = bootstrapVM.chatVM
        let bvm = bootstrapVM
        Task {
            var p = self.preview
            await chatVM.sendChatMessageToAPI(
                text,
                preview: &p
            )
            bvm.updateShellBanner(preview: &p)
            self.preview = p
        }
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

private extension PawCoreManager {
    static var defaultDevelopmentDeviceName: String {
#if os(iOS)
        UIDevice.current.name
#else
        "Development Device"
#endif
    }

    func normalized(_ value: String?) -> String? {
        guard let value else { return nil }
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }
}
