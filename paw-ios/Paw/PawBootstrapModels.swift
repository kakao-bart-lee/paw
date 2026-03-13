import Foundation

enum PawAuthStep: String, CaseIterable {
    case authMethodSelect = "AuthMethodSelect"
    case phoneInput = "PhoneInput"
    case otpVerify = "OtpVerify"
    case deviceName = "DeviceName"
    case usernameSetup = "UsernameSetup"
    case authenticated = "Authenticated"
}

struct PawBootstrapPreview {
    var bridgeStatus: String
    var auth: PawAuthPreview
    var runtime: PawRuntimePreview
    var storage: PawStoragePreview
    var push: PawPushPreview
    var lifecycle: PawLifecyclePreview
}

struct PawAuthPreview {
    var step: PawAuthStep
    var phone: String
    var deviceName: String
    var username: String
    var discoverableByPhone: Bool
    var hasSessionToken: Bool
    var hasAccessToken: Bool
    var hasRefreshToken: Bool
    var isLoading: Bool
    var error: String?
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

struct PawStoredTokens: Equatable {
    var sessionToken: String?
    var accessToken: String?
    var refreshToken: String?
}

protocol PawTokenVault {
    func loadTokens() -> PawStoredTokens
    func save(tokens: PawStoredTokens) -> Bool
    func clearTokens() -> Bool
    func storagePreview(hasDeviceKey: Bool) -> PawStoragePreview
}

protocol PawDeviceKeyStore {
    func hasDeviceKey() -> Bool
    func saveDeviceKey(_ data: Data) -> Bool
    func clearDeviceKey() -> Bool
}

protocol PawPushRegistrar {
    func currentState() -> PawPushPreview
    func register(token: String) -> PawPushPreview
    func unregister() -> PawPushPreview
}

protocol PawKeyValueSecureStore {
    func data(forKey key: String) -> Data?
    @discardableResult
    func set(_ data: Data, forKey key: String) -> Bool
    @discardableResult
    func removeValue(forKey key: String) -> Bool
}

final class PawInMemorySecureStore: PawKeyValueSecureStore {
    private var values: [String: Data] = [:]

    func data(forKey key: String) -> Data? {
        values[key]
    }

    func set(_ data: Data, forKey key: String) -> Bool {
        values[key] = data
        return true
    }

    func removeValue(forKey key: String) -> Bool {
        values.removeValue(forKey: key)
        return true
    }
}
