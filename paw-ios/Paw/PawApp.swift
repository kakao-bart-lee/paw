import SwiftUI

@main
struct PawApp: App {
    @StateObject private var coreManager = PawApp.makeCoreManager()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(coreManager)
        }
    }
}

private extension PawApp {
    static func makeCoreManager() -> PawCoreManager {
        let manager: PawCoreManager
#if DEBUG
        let environment = ProcessInfo.processInfo.environment
        guard environment["PAW_UI_TEST_MODE"] == "1" else {
            manager = PawCoreManager()
            if !manager.preview.auth.hasAccessToken {
                manager.devQuickLogin()
                manager.refresh()
            }
            return manager
        }

        let store = PawInMemorySecureStore()
        let tokenVault = PawKeychainTokenVault(secureStore: store)
        let deviceKeyStore = PawKeychainDeviceKeyStore(secureStore: store)
        let pushRegistrar = PawApnsPushRegistrar(secureStore: store)

        manager = PawCoreManager(
            tokenVault: tokenVault,
            deviceKeyStore: deviceKeyStore,
            pushRegistrar: pushRegistrar
        )
#else
        manager = PawCoreManager()
#endif
        return manager
    }
}
