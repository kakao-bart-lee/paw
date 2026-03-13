import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        NavigationStack {
            List {
                Section("App Shell") {
                    Label("SwiftUI bootstrap is wired", systemImage: "checkmark.seal.fill")
                    Label("Ready for auth/chat/search/settings flows", systemImage: "square.grid.2x2")
                }

                Section("Core") {
                    LabeledContent("Bindings", value: coreManager.bindingsStatus)
                    LabeledContent("Artifacts", value: coreManager.artifactsDirectory)
                }
            }
            .navigationTitle("Paw")
        }
    }
}
