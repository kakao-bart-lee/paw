import Foundation

@MainActor
final class PawCoreManager: ObservableObject {
    @Published private(set) var bindingsStatus = "Bindings contract ready"
    @Published private(set) var preview = PawBootstrapPreview(
        bridgeStatus: "Swift contract preview ready",
        auth: PawAuthPreview(
            step: "AuthMethodSelect",
            discoverableByPhone: false,
            hasAccessToken: false
        ),
        runtime: PawRuntimePreview(
            connectionState: "Disconnected",
            cursorCount: 0,
            activeStreamCount: 0
        ),
        storage: PawStoragePreview(
            provider: "Keychain (planned)",
            availability: "Contract ready"
        ),
        push: PawPushPreview(
            status: "Unregistered",
            platform: "apns"
        ),
        lifecycle: PawLifecyclePreview(
            activeHints: ["ReconnectSocket", "RefreshPushToken"],
            backgroundHints: ["PauseRealtime", "FlushAcks", "PersistDrafts"]
        )
    )

    var artifactsDirectory: String {
        "PawCore/Artifacts"
    }

    func previewPhoneInput() {
        preview.auth.step = "PhoneInput"
    }

    func resetPreview() {
        preview.auth.step = "AuthMethodSelect"
    }
}
