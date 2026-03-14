import Foundation

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
