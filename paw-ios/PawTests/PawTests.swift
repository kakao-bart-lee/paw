import XCTest
@testable import Paw

final class PawTests: XCTestCase {
    @MainActor
    func testArtifactsDirectoryMatchesGeneratedOutputLocation() {
        XCTAssertEqual(PawCoreManager().artifactsDirectory, "PawCore/Artifacts")
    }

    func testTokenVaultRoundTripAndClear() {
        let store = PawInMemorySecureStore()
        let vault = PawKeychainTokenVault(secureStore: store)
        let tokens = PawStoredTokens(sessionToken: "session", accessToken: "access", refreshToken: "refresh")

        XCTAssertTrue(vault.save(tokens: tokens))
        XCTAssertEqual(vault.loadTokens(), tokens)
        XCTAssertTrue(vault.clearTokens())
        XCTAssertEqual(vault.loadTokens(), PawStoredTokens(sessionToken: nil, accessToken: nil, refreshToken: nil))
    }

    func testDeviceKeyStoreRoundTrip() {
        let store = PawInMemorySecureStore()
        let keyStore = PawKeychainDeviceKeyStore(secureStore: store)

        XCTAssertFalse(keyStore.hasDeviceKey())
        XCTAssertTrue(keyStore.saveDeviceKey(Data("device-key".utf8)))
        XCTAssertTrue(keyStore.hasDeviceKey())
        XCTAssertTrue(keyStore.clearDeviceKey())
        XCTAssertFalse(keyStore.hasDeviceKey())
    }

    func testApnsRegistrarTracksRegistrationState() {
        let store = PawInMemorySecureStore()
        let registrar = PawApnsPushRegistrar(secureStore: store)

        XCTAssertEqual(registrar.currentState().status, "Unregistered")
        XCTAssertEqual(registrar.register(token: "apns-123").token, "apns-123")
        XCTAssertEqual(registrar.unregister().status, "Unregistered")
    }

    func testLifecycleHintsMatchContract() {
        XCTAssertEqual(PawCoreManager.lifecycleHints(for: "Active"), ["ReconnectSocket", "RefreshPushToken"])
        XCTAssertEqual(PawCoreManager.lifecycleHints(for: "Background"), ["PauseRealtime", "FlushAcks", "PersistDrafts"])
    }

    @MainActor
    func testBootstrapRestoresStoredAccessToken() {
        let store = PawInMemorySecureStore()
        let vault = PawKeychainTokenVault(secureStore: store)
        _ = vault.save(tokens: PawStoredTokens(sessionToken: "session", accessToken: "access", refreshToken: nil))

        let manager = PawCoreManager(
            tokenVault: vault,
            deviceKeyStore: PawKeychainDeviceKeyStore(secureStore: store),
            pushRegistrar: PawApnsPushRegistrar(secureStore: store)
        )

        XCTAssertEqual(manager.preview.auth.step, .authenticated)
        XCTAssertTrue(manager.preview.auth.hasAccessToken)
        XCTAssertEqual(manager.preview.runtime.connectionState, "Ready")
    }

    @MainActor
    func testAuthFlowTransitionsToAuthenticatedAndPersistsState() {
        let store = PawInMemorySecureStore()
        let vault = PawKeychainTokenVault(secureStore: store)
        let manager = PawCoreManager(
            tokenVault: vault,
            deviceKeyStore: PawKeychainDeviceKeyStore(secureStore: store),
            pushRegistrar: PawApnsPushRegistrar(secureStore: store)
        )

        manager.startPhoneInput()
        manager.submitPhone("+82 10-7777-0001", discoverableByPhone: true)
        manager.verifyOtp("654321")
        manager.submitDeviceName("QA iPhone")
        manager.submitUsername("pawtester")
        manager.registerForPush(token: "apns-demo-token")
        manager.applyLifecycle(state: "Active")

        XCTAssertEqual(manager.preview.auth.step, .authenticated)
        XCTAssertEqual(manager.preview.auth.phone, "+82 10-7777-0001")
        XCTAssertEqual(manager.preview.auth.deviceName, "QA iPhone")
        XCTAssertEqual(manager.preview.auth.username, "pawtester")
        XCTAssertTrue(manager.preview.auth.hasAccessToken)
        XCTAssertTrue(manager.preview.storage.hasDeviceKey)
        XCTAssertEqual(manager.preview.push.status, "Registered")
        XCTAssertEqual(manager.preview.runtime.connectionState, "Connected")
        XCTAssertEqual(vault.loadTokens().accessToken, "access-ios-demo")
    }
}
