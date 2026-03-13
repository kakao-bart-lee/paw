import SwiftUI

@main
struct PawApp: App {
    @StateObject private var coreManager = PawCoreManager()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(coreManager)
        }
    }
}
