import Foundation

/// Owns auth step progression and auth-related state mutations.
/// All state is managed through the shared PawBootstrapPreview reference
/// to maintain backward compatibility with PawCoreManager's @Published preview.
@MainActor
final class AuthViewModel {
    private let tokenVault: PawTokenVault
    private let deviceKeyStore: PawDeviceKeyStore

    init(tokenVault: PawTokenVault, deviceKeyStore: PawDeviceKeyStore) {
        self.tokenVault = tokenVault
        self.deviceKeyStore = deviceKeyStore
    }

    func startPhoneInput(_ preview: inout PawBootstrapPreview) {
        preview.auth.step = .phoneInput
        preview.auth.error = nil
    }

    func submitPhone(
        _ phone: String = "+82 10-5555-0101",
        discoverableByPhone: Bool = true,
        preview: inout PawBootstrapPreview
    ) {
        preview.auth.phone = phone
        preview.auth.discoverableByPhone = discoverableByPhone
        preview.auth.step = .otpVerify
        preview.auth.error = nil
    }

    func verifyOtp(
        _ code: String = "123456",
        preview: inout PawBootstrapPreview
    ) {
        guard code.count >= 6 else {
            preview.auth.error = "OTP must be 6 digits"
            return
        }

        let tokens = PawStoredTokens(
            sessionToken: "session-ios-demo",
            accessToken: "access-ios-demo",
            refreshToken: "refresh-ios-demo"
        )
        guard tokenVault.save(tokens: tokens) else {
            preview.auth.error = "Failed to persist session in Keychain"
            return
        }

        preview.auth.hasSessionToken = true
        preview.auth.hasAccessToken = true
        preview.auth.hasRefreshToken = true
        preview.auth.step = .deviceName
        preview.runtime.connectionState = "Bootstrapping"
        preview.auth.error = nil
    }

    func submitDeviceName(
        _ deviceName: String = "Haruna's iPhone",
        preview: inout PawBootstrapPreview
    ) {
        preview.auth.deviceName = deviceName
        preview.auth.step = .usernameSetup
        preview.auth.error = nil
        if !deviceKeyStore.hasDeviceKey() {
            _ = deviceKeyStore.saveDeviceKey(Data(deviceName.utf8))
            preview.storage.hasDeviceKey = deviceKeyStore.hasDeviceKey()
        }
    }

    func submitUsername(
        _ username: String = "haruna",
        preview: inout PawBootstrapPreview
    ) {
        preview.auth.username = username
        preview.auth.step = .authenticated
        preview.runtime.connectionState = "Connected"
        preview.auth.error = nil
    }

    func skipUsername(preview: inout PawBootstrapPreview, effectiveUsername: String) {
        preview.auth.step = .authenticated
        preview.runtime.connectionState = "Connected"
        preview.auth.error = nil
    }

    func logout(preview: inout PawBootstrapPreview, pushRegistrar: PawPushRegistrar) {
        _ = tokenVault.clearTokens()
        _ = pushRegistrar.unregister()
        preview.auth = Self.makeAuthPreview(
            tokens: PawStoredTokens(sessionToken: nil, accessToken: nil, refreshToken: nil)
        )
        preview.runtime.connectionState = "Disconnected"
        preview.push = pushRegistrar.currentState()
    }

    static func makeAuthPreview(tokens: PawStoredTokens) -> PawAuthPreview {
        PawAuthPreview(
            step: tokens.accessToken == nil ? .authMethodSelect : .authenticated,
            phone: "",
            deviceName: "",
            username: "",
            discoverableByPhone: false,
            hasSessionToken: tokens.sessionToken != nil,
            hasAccessToken: tokens.accessToken != nil,
            hasRefreshToken: tokens.refreshToken != nil,
            isLoading: false,
            error: nil
        )
    }
}
