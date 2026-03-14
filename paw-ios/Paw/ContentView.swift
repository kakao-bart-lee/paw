import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        ZStack {
            LinearGradient(
                colors: [PawTheme.surface1, PawTheme.background],
                startPoint: .top,
                endPoint: .bottom
            )
            .ignoresSafeArea()

            PawBootstrapView()
                .environmentObject(coreManager)
        }
    }
}
