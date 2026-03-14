import SwiftUI

struct BootstrapView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    private var shouldShowMainShell: Bool {
        coreManager.preview.auth.hasAccessToken || coreManager.preview.auth.step == .authenticated
    }

    var body: some View {
        Group {
            if shouldShowMainShell {
                ChatView()
                    .environmentObject(coreManager)
            } else {
                AuthView()
                    .environmentObject(coreManager)
            }
        }
        .animation(.easeInOut(duration: 0.22), value: shouldShowMainShell)
    }
}
