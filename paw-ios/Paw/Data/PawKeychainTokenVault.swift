import Foundation

final class PawKeychainTokenVault: PawTokenVault {
    private let secureStore: PawKeyValueSecureStore

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore()) {
        self.secureStore = secureStore
    }

    func loadTokens() -> PawStoredTokens {
        PawStoredTokens(
            sessionToken: decode("dev.paw.auth.session"),
            accessToken: decode("dev.paw.auth.access"),
            refreshToken: decode("dev.paw.auth.refresh")
        )
    }

    func save(tokens: PawStoredTokens) -> Bool {
        persist(tokens.sessionToken, forKey: "dev.paw.auth.session")
            && persist(tokens.accessToken, forKey: "dev.paw.auth.access")
            && persist(tokens.refreshToken, forKey: "dev.paw.auth.refresh")
    }

    func clearTokens() -> Bool {
        secureStore.removeValue(forKey: "dev.paw.auth.session")
            && secureStore.removeValue(forKey: "dev.paw.auth.access")
            && secureStore.removeValue(forKey: "dev.paw.auth.refresh")
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
