import Foundation

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
