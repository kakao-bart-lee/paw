import Foundation

final class PawApnsPushRegistrar: PawPushRegistrar {
    private let secureStore: PawKeyValueSecureStore
    private let now: () -> Date
    private var lastError: String?

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore(), now: @escaping () -> Date = Date.init) {
        self.secureStore = secureStore
        self.now = now
    }

    func currentState() -> PawPushPreview {
        let token = secureStore.data(forKey: "dev.paw.push.token")
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
        _ = secureStore.set(Data(token.utf8), forKey: "dev.paw.push.token")
        lastError = nil
        return currentState()
    }

    func unregister() -> PawPushPreview {
        _ = secureStore.removeValue(forKey: "dev.paw.push.token")
        lastError = nil
        return currentState()
    }
}
