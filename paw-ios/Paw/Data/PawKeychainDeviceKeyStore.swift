import Foundation

final class PawKeychainDeviceKeyStore: PawDeviceKeyStore {
    private let secureStore: PawKeyValueSecureStore

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore()) {
        self.secureStore = secureStore
    }

    func hasDeviceKey() -> Bool {
        secureStore.data(forKey: "dev.paw.device.key") != nil
    }

    func saveDeviceKey(_ data: Data) -> Bool {
        secureStore.set(data, forKey: "dev.paw.device.key")
    }

    func clearDeviceKey() -> Bool {
        secureStore.removeValue(forKey: "dev.paw.device.key")
    }
}
