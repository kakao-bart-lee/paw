import Foundation

protocol PawTokenVault {
    func loadTokens() -> PawStoredTokens
    func save(tokens: PawStoredTokens) -> Bool
    func clearTokens() -> Bool
    func storagePreview(hasDeviceKey: Bool) -> PawStoragePreview
}
