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
#if DEBUG
        let environment = ProcessInfo.processInfo.environment
        guard environment["PAW_UI_TEST_MODE"] == "1" else {
            return PawCoreManager()
        }

        let store = PawInMemorySecureStore()
        let tokenVault = PawKeychainTokenVault(secureStore: store)
        let deviceKeyStore = PawKeychainDeviceKeyStore(secureStore: store)
        let pushRegistrar = PawApnsPushRegistrar(secureStore: store)

        if environment["PAW_UI_TEST_PRELOAD_AUTH"] == "1" {
            _ = tokenVault.save(tokens: PawStoredTokens(
                sessionToken: "session-ui-test",
                accessToken: "access-ui-test",
                refreshToken: "refresh-ui-test"
            ))
            _ = deviceKeyStore.saveDeviceKey(Data("ui-test-device".utf8))
        }

        return PawCoreManager(
            tokenVault: tokenVault,
            deviceKeyStore: deviceKeyStore,
            pushRegistrar: pushRegistrar
        )
#else
        return PawCoreManager()
#endif
    }
}
