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
        XCTAssertEqual(manager.preview.shellBanner, "Bootstrap Crew · Ready · push pending")
        XCTAssertEqual(manager.preview.conversations.count, 2)
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
        XCTAssertEqual(manager.selectedConversation?.id, "conv-bootstrap")
        XCTAssertEqual(manager.preview.messages.first?.author, "System")
    }

    @MainActor
    func testChatShellUnlocksAfterAuthAndAppendsAgentReply() {
        let store = PawInMemorySecureStore()
        let manager = PawCoreManager(
            tokenVault: PawKeychainTokenVault(secureStore: store),
            deviceKeyStore: PawKeychainDeviceKeyStore(secureStore: store),
            pushRegistrar: PawApnsPushRegistrar(secureStore: store),
            now: { Date(timeIntervalSince1970: 1_710_000_000) }
        )

        manager.sendChatMessage()
        XCTAssertEqual(manager.preview.auth.error, "Finish auth flow before using chat shell")

        manager.submitPhone()
        manager.verifyOtp()
        manager.submitDeviceName()
        manager.submitUsername("chatuser")
        manager.selectConversation("conv-agent")
        manager.sendChatMessage("How is runtime?")

        XCTAssertEqual(manager.preview.selectedConversationID, "conv-agent")
        XCTAssertEqual(manager.preview.runtime.activeStreamCount, 0)
        XCTAssertEqual(manager.preview.messages.last?.author, "Paw Agent")
        XCTAssertTrue(manager.preview.messages.last?.body.contains("Runtime live") == true)
        XCTAssertEqual(manager.preview.conversations.first(where: { $0.id == "conv-agent" })?.unreadCount, 0)
    }

    @MainActor
    func testLogoutRelocksConversationShell() {
        let store = PawInMemorySecureStore()
        let manager = PawCoreManager(
            tokenVault: PawKeychainTokenVault(secureStore: store),
            deviceKeyStore: PawKeychainDeviceKeyStore(secureStore: store),
            pushRegistrar: PawApnsPushRegistrar(secureStore: store)
        )

        manager.submitPhone()
        manager.verifyOtp()
        manager.submitDeviceName()
        manager.submitUsername()
        manager.logout()

        XCTAssertFalse(manager.preview.auth.hasAccessToken)
        XCTAssertEqual(manager.preview.runtime.connectionState, "Disconnected")
        XCTAssertEqual(manager.preview.shellBanner, "Authenticate to unlock conversations + chat runtime shell.")
        XCTAssertEqual(manager.preview.messages.first?.body, "No stored token found. Auth flow starts from phone input.")
    }

    @MainActor
    func testSkipUsernameAuthenticatesWithFallbackChatIdentity() {
        let store = PawInMemorySecureStore()
        let manager = PawCoreManager(
            tokenVault: PawKeychainTokenVault(secureStore: store),
            deviceKeyStore: PawKeychainDeviceKeyStore(secureStore: store),
            pushRegistrar: PawApnsPushRegistrar(secureStore: store)
        )

        manager.submitPhone()
        manager.verifyOtp()
        manager.submitDeviceName("Design QA")
        manager.skipUsername()

        XCTAssertEqual(manager.preview.auth.step, .authenticated)
        XCTAssertEqual(manager.preview.auth.username, "")
        XCTAssertTrue(manager.preview.auth.hasAccessToken)
        XCTAssertEqual(manager.preview.runtime.connectionState, "Connected")
        XCTAssertEqual(manager.preview.messages.first?.author, "System")
        XCTAssertEqual(manager.preview.messages.last?.author, "Paw Agent")
        XCTAssertEqual(manager.selectedConversation?.title, "Bootstrap Crew")
    }

    @MainActor
    func testRefreshRebuildsBannerAfterPushStateChanges() {
        let store = PawInMemorySecureStore()
        let manager = PawCoreManager(
            tokenVault: PawKeychainTokenVault(secureStore: store),
            deviceKeyStore: PawKeychainDeviceKeyStore(secureStore: store),
            pushRegistrar: PawApnsPushRegistrar(secureStore: store)
        )

        manager.submitPhone()
        manager.verifyOtp()
        manager.submitDeviceName()
        manager.submitUsername("refresh-user")
        manager.registerForPush(token: "apns-refresh")
        XCTAssertEqual(manager.preview.shellBanner, "Bootstrap Crew · Connected · push registered")

        manager.unregisterPush()
        manager.refresh()

        XCTAssertEqual(manager.preview.push.status, "Unregistered")
        XCTAssertEqual(manager.preview.shellBanner, "Bootstrap Crew · Connected · push unregistered")
    }
}
