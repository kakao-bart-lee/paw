import Foundation

@MainActor
final class PawCoreManager: ObservableObject {
    @Published private(set) var bindingsStatus = "Bindings placeholder ready"

    var artifactsDirectory: String {
        "PawCore/Artifacts"
    }
}
