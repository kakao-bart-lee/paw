import Foundation

/// Owns auth step progression and auth-related state mutations.
/// All state is managed through the shared PawBootstrapPreview reference
/// to maintain backward compatibility with PawCoreManager's @Published preview.
@MainActor
final class AuthViewModel {
    private let tokenVault: PawTokenVault
    private let deviceKeyStore: PawDeviceKeyStore
    let apiClient: PawApiClient

    /// The phone number entered during phoneInput, retained for OTP verification.
    private(set) var currentPhone: String = ""
    /// Session token received after OTP verification, used for device registration.
    private(set) var sessionToken: String?

    init(
        tokenVault: PawTokenVault,
        deviceKeyStore: PawDeviceKeyStore,
        apiClient: PawApiClient = PawApiClient()
    ) {
        self.tokenVault = tokenVault
        self.deviceKeyStore = deviceKeyStore
        self.apiClient = apiClient
    }

    func startPhoneInput(_ preview: inout PawBootstrapPreview) {
        preview.auth.step = .phoneInput
        preview.auth.error = nil
    }

    // MARK: - Async API-backed methods (used by views)

    func submitPhoneAsync(
        _ phone: String,
        discoverableByPhone: Bool = true,
        preview: inout PawBootstrapPreview
    ) async {
        let trimmed = phone.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            preview.auth.error = "Please enter a phone number"
            return
        }

        currentPhone = trimmed
        preview.auth.phone = trimmed
        preview.auth.discoverableByPhone = discoverableByPhone
        preview.auth.isLoading = true
        preview.auth.error = nil

        do {
            _ = try await apiClient.requestOtp(phone: trimmed)
            preview.auth.step = .otpVerify
        } catch {
            preview.auth.error = error.localizedDescription
        }

        preview.auth.isLoading = false
    }

    func verifyOtpAsync(
        _ code: String,
        preview: inout PawBootstrapPreview
    ) async {
        let trimmed = code.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.count >= 6 else {
            preview.auth.error = "OTP must be 6 digits"
            return
        }

        preview.auth.isLoading = true
        preview.auth.error = nil

        do {
            let result = try await apiClient.verifyOtp(phone: currentPhone, code: trimmed)
            let token = result["session_token"] as? String ?? ""

            guard !token.isEmpty else {
                preview.auth.error = "Server did not return a session token"
                preview.auth.isLoading = false
                return
            }

            sessionToken = token

            let tokens = PawStoredTokens(
                sessionToken: token,
                accessToken: nil,
                refreshToken: nil
            )
            guard tokenVault.save(tokens: tokens) else {
                preview.auth.error = "Failed to persist session in Keychain"
                preview.auth.isLoading = false
                return
            }

            preview.auth.hasSessionToken = true
            preview.auth.step = .deviceName
            preview.runtime.connectionState = "Bootstrapping"
        } catch {
            preview.auth.error = error.localizedDescription
        }

        preview.auth.isLoading = false
    }

    func submitDeviceNameAsync(
        _ deviceName: String,
        preview: inout PawBootstrapPreview
    ) async {
        let trimmed = deviceName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            preview.auth.error = "Please enter a device name"
            return
        }

        guard let session = sessionToken, !session.isEmpty else {
            preview.auth.error = "Session expired. Please restart login."
            return
        }

        preview.auth.isLoading = true
        preview.auth.error = nil

        if !deviceKeyStore.hasDeviceKey() {
            _ = deviceKeyStore.saveDeviceKey(Data(trimmed.utf8))
        }
        let publicKeyBase64 = Data(trimmed.utf8).base64EncodedString()

        do {
            let result = try await apiClient.registerDevice(
                sessionToken: session,
                deviceName: trimmed,
                ed25519PublicKey: publicKeyBase64
            )

            let accessToken = result["access_token"] as? String ?? ""
            let refreshToken = result["refresh_token"] as? String ?? ""

            guard !accessToken.isEmpty else {
                preview.auth.error = "Server did not return access credentials"
                preview.auth.isLoading = false
                return
            }

            let tokens = PawStoredTokens(
                sessionToken: session,
                accessToken: accessToken,
                refreshToken: refreshToken
            )
            guard tokenVault.save(tokens: tokens) else {
                preview.auth.error = "Failed to persist tokens in Keychain"
                preview.auth.isLoading = false
                return
            }

            await apiClient.setAccessToken(accessToken)

            preview.auth.deviceName = trimmed
            preview.auth.hasAccessToken = true
            preview.auth.hasRefreshToken = !refreshToken.isEmpty
            preview.auth.step = .usernameSetup
            preview.storage.hasDeviceKey = deviceKeyStore.hasDeviceKey()
        } catch {
            preview.auth.error = error.localizedDescription
        }

        preview.auth.isLoading = false
    }

    func submitUsernameAsync(
        _ username: String,
        discoverableByPhone: Bool,
        preview: inout PawBootstrapPreview
    ) async {
        let trimmed = username.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            preview.auth.error = "Please enter a username"
            return
        }

        preview.auth.isLoading = true
        preview.auth.error = nil

        do {
            _ = try await apiClient.updateMe(
                username: trimmed,
                discoverableByPhone: discoverableByPhone
            )
            preview.auth.username = trimmed
            preview.auth.discoverableByPhone = discoverableByPhone
            preview.auth.step = .authenticated
            preview.runtime.connectionState = "Connected"
        } catch {
            preview.auth.error = error.localizedDescription
        }

        preview.auth.isLoading = false
    }

    // MARK: - Synchronous methods (used by tests and backward-compat)

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
        currentPhone = ""
        sessionToken = nil
        Task { await apiClient.setAccessToken(nil) }
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
