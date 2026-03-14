import Foundation

protocol PawDeviceKeyStore {
    func hasDeviceKey() -> Bool
    func saveDeviceKey(_ data: Data) -> Bool
    func clearDeviceKey() -> Bool
}
