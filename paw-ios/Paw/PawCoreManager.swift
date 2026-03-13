import Foundation
import Security

private enum PawSecureStoreKey {
    static let sessionToken = "dev.paw.auth.session"
    static let accessToken = "dev.paw.auth.access"
    static let refreshToken = "dev.paw.auth.refresh"
    static let deviceKey = "dev.paw.device.key"
    static let pushToken = "dev.paw.push.token"
}

final class PawKeychainStore: PawKeyValueSecureStore {
    func data(forKey key: String) -> Data? {
        var item: CFTypeRef?
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitOne,
        ]
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        guard status == errSecSuccess else {
            return nil
        }
        return item as? Data
    }

    @discardableResult
    func set(_ data: Data, forKey key: String) -> Bool {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
        ]
        let attributes: [CFString: Any] = [kSecValueData: data]
        let updateStatus = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        if updateStatus == errSecSuccess {
            return true
        }

        var insert = query
        insert[kSecValueData] = data
        insert[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        return SecItemAdd(insert as CFDictionary, nil) == errSecSuccess
    }

    @discardableResult
    func removeValue(forKey key: String) -> Bool {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
        ]
        let status = SecItemDelete(query as CFDictionary)
        return status == errSecSuccess || status == errSecItemNotFound
    }
}

final class PawKeychainTokenVault: PawTokenVault {
    private let secureStore: PawKeyValueSecureStore

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore()) {
        self.secureStore = secureStore
    }

    func loadTokens() -> PawStoredTokens {
        PawStoredTokens(
            sessionToken: decode(PawSecureStoreKey.sessionToken),
            accessToken: decode(PawSecureStoreKey.accessToken),
            refreshToken: decode(PawSecureStoreKey.refreshToken)
        )
    }

    func save(tokens: PawStoredTokens) -> Bool {
        persist(tokens.sessionToken, forKey: PawSecureStoreKey.sessionToken)
            && persist(tokens.accessToken, forKey: PawSecureStoreKey.accessToken)
            && persist(tokens.refreshToken, forKey: PawSecureStoreKey.refreshToken)
    }

    func clearTokens() -> Bool {
        secureStore.removeValue(forKey: PawSecureStoreKey.sessionToken)
            && secureStore.removeValue(forKey: PawSecureStoreKey.accessToken)
            && secureStore.removeValue(forKey: PawSecureStoreKey.refreshToken)
    }

    func storagePreview(hasDeviceKey: Bool) -> PawStoragePreview {
        PawStoragePreview(
            provider: "Keychain",
            availability: "Available",
            hasDeviceKey: hasDeviceKey
        )
    }

    private func decode(_ key: String) -> String? {
        guard let data = secureStore.data(forKey: key) else {
            return nil
        }
        return String(data: data, encoding: .utf8)
    }

    private func persist(_ value: String?, forKey key: String) -> Bool {
        guard let value else {
            return secureStore.removeValue(forKey: key)
        }
        return secureStore.set(Data(value.utf8), forKey: key)
    }
}

final class PawKeychainDeviceKeyStore: PawDeviceKeyStore {
    private let secureStore: PawKeyValueSecureStore

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore()) {
        self.secureStore = secureStore
    }

    func hasDeviceKey() -> Bool {
        secureStore.data(forKey: PawSecureStoreKey.deviceKey) != nil
    }

    func saveDeviceKey(_ data: Data) -> Bool {
        secureStore.set(data, forKey: PawSecureStoreKey.deviceKey)
    }

    func clearDeviceKey() -> Bool {
        secureStore.removeValue(forKey: PawSecureStoreKey.deviceKey)
    }
}

final class PawApnsPushRegistrar: PawPushRegistrar {
    private let secureStore: PawKeyValueSecureStore
    private let now: () -> Date
    private var lastError: String?

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore(), now: @escaping () -> Date = Date.init) {
        self.secureStore = secureStore
        self.now = now
    }

    func currentState() -> PawPushPreview {
        let token = secureStore.data(forKey: PawSecureStoreKey.pushToken)
            .flatMap { String(data: $0, encoding: .utf8) }
        return PawPushPreview(
            status: token == nil ? "Unregistered" : "Registered",
            platform: "apns",
            token: token,
            lastError: lastError,
            lastUpdatedMs: Int(now().timeIntervalSince1970 * 1_000)
        )
    }

    func register(token: String) -> PawPushPreview {
        guard !token.isEmpty else {
            lastError = "Empty APNs token"
            return currentState()
        }
        _ = secureStore.set(Data(token.utf8), forKey: PawSecureStoreKey.pushToken)
        lastError = nil
        return currentState()
    }

    func unregister() -> PawPushPreview {
        _ = secureStore.removeValue(forKey: PawSecureStoreKey.pushToken)
        lastError = nil
        return currentState()
    }
}

@MainActor
final class PawCoreManager: ObservableObject {
    @Published private(set) var bindingsStatus = "Native iOS bootstrap ready"
    @Published private(set) var preview: PawBootstrapPreview

    private let tokenVault: PawTokenVault
    private let deviceKeyStore: PawDeviceKeyStore
    private let pushRegistrar: PawPushRegistrar
    private let now: () -> Date

    init(
        tokenVault: PawTokenVault = PawKeychainTokenVault(),
        deviceKeyStore: PawDeviceKeyStore = PawKeychainDeviceKeyStore(),
        pushRegistrar: PawPushRegistrar = PawApnsPushRegistrar(),
        now: @escaping () -> Date = Date.init
    ) {
        self.tokenVault = tokenVault
        self.deviceKeyStore = deviceKeyStore
        self.pushRegistrar = pushRegistrar
        self.now = now

        let hasDeviceKey = deviceKeyStore.hasDeviceKey()
        let tokens = tokenVault.loadTokens()
        self.preview = PawBootstrapPreview(
            bridgeStatus: "Keychain + APNs adapters wired",
            auth: PawCoreManager.makeAuthPreview(tokens: tokens),
            runtime: PawRuntimePreview(
                connectionState: tokens.accessToken == nil ? "Disconnected" : "Ready",
                cursorCount: 0,
                activeStreamCount: 0
            ),
            storage: tokenVault.storagePreview(hasDeviceKey: hasDeviceKey),
            push: pushRegistrar.currentState(),
            lifecycle: PawLifecyclePreview(
                activeHints: Self.lifecycleHints(for: "Active"),
                backgroundHints: Self.lifecycleHints(for: "Background"),
                currentState: "Launching"
            )
        )
    }

    var artifactsDirectory: String {
        "PawCore/Artifacts"
    }

    func startPhoneInput() {
        preview.auth.step = .phoneInput
        preview.auth.error = nil
    }

    func submitPhone(_ phone: String = "+82 10-5555-0101", discoverableByPhone: Bool = true) {
        preview.auth.phone = phone
        preview.auth.discoverableByPhone = discoverableByPhone
        preview.auth.step = .otpVerify
        preview.auth.error = nil
    }

    func verifyOtp(_ code: String = "123456") {
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

    func submitDeviceName(_ deviceName: String = "Haruna's iPhone") {
        preview.auth.deviceName = deviceName
        preview.auth.step = .usernameSetup
        preview.auth.error = nil
        if !deviceKeyStore.hasDeviceKey() {
            _ = deviceKeyStore.saveDeviceKey(Data(deviceName.utf8))
            preview.storage.hasDeviceKey = deviceKeyStore.hasDeviceKey()
        }
    }

    func submitUsername(_ username: String = "haruna") {
        preview.auth.username = username
        preview.auth.step = .authenticated
        preview.runtime.connectionState = "Connected"
        preview.auth.error = nil
    }

    func logout() {
        _ = tokenVault.clearTokens()
        _ = pushRegistrar.unregister()
        preview.auth = Self.makeAuthPreview(tokens: PawStoredTokens(sessionToken: nil, accessToken: nil, refreshToken: nil))
        preview.runtime.connectionState = "Disconnected"
        preview.push = pushRegistrar.currentState()
    }

    func registerForPush(token: String = "apns-demo-token") {
        preview.push = pushRegistrar.register(token: token)
    }

    func unregisterPush() {
        preview.push = pushRegistrar.unregister()
    }

    func applyLifecycle(state: String) {
        preview.lifecycle.currentState = state
        switch state {
        case "Active":
            preview.lifecycle.activeHints = Self.lifecycleHints(for: state)
            preview.runtime.connectionState = preview.auth.hasAccessToken ? "Connected" : "Disconnected"
        case "Background":
            preview.lifecycle.backgroundHints = Self.lifecycleHints(for: state)
            preview.runtime.connectionState = "Background"
        default:
            break
        }
    }

    private static func makeAuthPreview(tokens: PawStoredTokens) -> PawAuthPreview {
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
