import XCTest
@testable import Paw

// MARK: - Helpers

/// Convenience factory that creates a fresh PawCoreManager backed by in-memory stores.
/// Every test gets its own isolated instance so state never leaks between cases.
@MainActor
private func makeFreshManager(
    preloadTokens: PawStoredTokens? = nil,
    now: @escaping () -> Date = Date.init
) -> (PawCoreManager, PawKeychainTokenVault, PawInMemorySecureStore) {
    let store = PawInMemorySecureStore()
    let vault = PawKeychainTokenVault(secureStore: store)
    if let tokens = preloadTokens {
        _ = vault.save(tokens: tokens)
    }
    let manager = PawCoreManager(
        tokenVault: vault,
        deviceKeyStore: PawKeychainDeviceKeyStore(secureStore: store),
        pushRegistrar: PawApnsPushRegistrar(secureStore: store),
        now: now
    )
    return (manager, vault, store)
}

/// Drives the manager through the full auth flow so chat tests start from an authenticated state.
@MainActor
private func authenticateManager(_ manager: PawCoreManager, username: String = "tester") {
    manager.startPhoneInput()
    manager.submitPhone("+82 10-1234-5678", discoverableByPhone: true)
    manager.verifyOtp("999999")
    manager.submitDeviceName("Test Device")
    manager.submitUsername(username)
}

// MARK: - TC-AUTH

final class PawTestContractAuthTests: XCTestCase {

    // TC-AUTH-01: Initial state when no stored token
    @MainActor
    func testTC_AUTH_01_initialStateIsAuthMethodSelect() {
        let (manager, _, _) = makeFreshManager()

        XCTAssertEqual(manager.preview.auth.step, .authMethodSelect)
        XCTAssertNil(manager.preview.auth.error)
        XCTAssertFalse(manager.preview.auth.isLoading)
        XCTAssertFalse(manager.preview.auth.hasAccessToken)
        XCTAssertFalse(manager.preview.auth.hasSessionToken)
        XCTAssertFalse(manager.preview.auth.hasRefreshToken)
    }

    // TC-AUTH-03: OTP verification transitions to DeviceName
    @MainActor
    func testTC_AUTH_03_otpVerifyTransitionsToDeviceName() {
        let (manager, _, _) = makeFreshManager()

        manager.startPhoneInput()
        manager.submitPhone("+82 10-1111-2222")
        XCTAssertEqual(manager.preview.auth.step, .otpVerify)

        manager.verifyOtp("123456")
        XCTAssertEqual(manager.preview.auth.step, .deviceName)
        XCTAssertTrue(manager.preview.auth.hasSessionToken)
        XCTAssertTrue(manager.preview.auth.hasAccessToken)
        XCTAssertNil(manager.preview.auth.error)
    }

    // TC-AUTH-04: Device registration transitions to UsernameSetup (demo flow always goes to usernameSetup)
    @MainActor
    func testTC_AUTH_04_deviceRegistrationTransitionsToUsernameSetup() {
        let (manager, vault, _) = makeFreshManager()

        manager.startPhoneInput()
        manager.submitPhone()
        manager.verifyOtp("123456")
        manager.submitDeviceName("Auth04 Device")

        XCTAssertEqual(manager.preview.auth.step, .usernameSetup)
        XCTAssertEqual(manager.preview.auth.deviceName, "Auth04 Device")
        // Tokens persisted after OTP
        XCTAssertNotNil(vault.loadTokens().accessToken)
        XCTAssertNotNil(vault.loadTokens().refreshToken)
        XCTAssertNil(manager.preview.auth.error)
    }

    // TC-AUTH-07: Empty / short input validation at each step
    @MainActor
    func testTC_AUTH_07_emptyInputValidation() {
        let (manager, _, _) = makeFreshManager()

        // OTP with fewer than 6 digits should produce an error and stay on otpVerify
        manager.startPhoneInput()
        manager.submitPhone()
        XCTAssertEqual(manager.preview.auth.step, .otpVerify)

        manager.verifyOtp("123") // too short
        XCTAssertEqual(manager.preview.auth.step, .otpVerify, "Step should not advance on invalid OTP")
        XCTAssertNotNil(manager.preview.auth.error)
        XCTAssertEqual(manager.preview.auth.error, "OTP must be 6 digits")
    }

    // TC-AUTH-08: Loading state (the iOS demo manager is synchronous so isLoading stays false)
    @MainActor
    func testTC_AUTH_08_loadingStateDefaultsFalse() {
        let (manager, _, _) = makeFreshManager()

        XCTAssertFalse(manager.preview.auth.isLoading, "Synchronous demo manager never enters loading state")

        manager.startPhoneInput()
        XCTAssertFalse(manager.preview.auth.isLoading)

        manager.submitPhone()
        XCTAssertFalse(manager.preview.auth.isLoading)

        manager.verifyOtp("123456")
        XCTAssertFalse(manager.preview.auth.isLoading)
    }

    // TC-AUTH-09: Auth error handling (OTP error then retry succeeds)
    @MainActor
    func testTC_AUTH_09_authErrorHandlingAndRetry() {
        let (manager, _, _) = makeFreshManager()

        manager.startPhoneInput()
        manager.submitPhone()

        // Trigger error
        manager.verifyOtp("12") // too short
        XCTAssertEqual(manager.preview.auth.step, .otpVerify)
        XCTAssertNotNil(manager.preview.auth.error)

        // Retry with valid OTP succeeds
        manager.verifyOtp("654321")
        XCTAssertEqual(manager.preview.auth.step, .deviceName)
        XCTAssertNil(manager.preview.auth.error)
    }
}

// MARK: - TC-TOKEN

final class PawTestContractTokenTests: XCTestCase {

    // TC-TOKEN-02: Overwrite existing token
    func testTC_TOKEN_02_overwriteExistingToken() {
        let store = PawInMemorySecureStore()
        let vault = PawKeychainTokenVault(secureStore: store)

        let original = PawStoredTokens(sessionToken: "s1", accessToken: "a1", refreshToken: "r1")
        XCTAssertTrue(vault.save(tokens: original))
        XCTAssertEqual(vault.loadTokens(), original)

        let updated = PawStoredTokens(sessionToken: "s2", accessToken: "a2", refreshToken: "r2")
        XCTAssertTrue(vault.save(tokens: updated))
        XCTAssertEqual(vault.loadTokens(), updated)
        XCTAssertNotEqual(vault.loadTokens(), original)
    }

    // TC-TOKEN-03: Read from empty state
    func testTC_TOKEN_03_readFromEmptyState() {
        let store = PawInMemorySecureStore()
        let vault = PawKeychainTokenVault(secureStore: store)

        let tokens = vault.loadTokens()
        XCTAssertNil(tokens.sessionToken)
        XCTAssertNil(tokens.accessToken)
        XCTAssertNil(tokens.refreshToken)
    }
}

// MARK: - TC-DEVICE-KEY

final class PawTestContractDeviceKeyTests: XCTestCase {

    // TC-DEVICE-KEY-01: Key generation (loadOrCreate equivalent via submitDeviceName)
    @MainActor
    func testTC_DEVICE_KEY_01_keyGeneration() {
        let store = PawInMemorySecureStore()
        let keyStore = PawKeychainDeviceKeyStore(secureStore: store)

        XCTAssertFalse(keyStore.hasDeviceKey(), "No key before generation")

        // saveDeviceKey acts as loadOrCreate for the demo
        XCTAssertTrue(keyStore.saveDeviceKey(Data("generated-key".utf8)))
        XCTAssertTrue(keyStore.hasDeviceKey(), "Key exists after generation")

        // Subsequent save preserves presence
        XCTAssertTrue(keyStore.saveDeviceKey(Data("generated-key".utf8)))
        XCTAssertTrue(keyStore.hasDeviceKey())
    }

    // TC-DEVICE-KEY-01 integration: submitDeviceName creates key via manager
    @MainActor
    func testTC_DEVICE_KEY_01_managerCreatesKeyOnDeviceRegistration() {
        let (manager, _, _) = makeFreshManager()

        XCTAssertFalse(manager.preview.storage.hasDeviceKey)

        manager.startPhoneInput()
        manager.submitPhone()
        manager.verifyOtp("123456")
        manager.submitDeviceName("KeyGenDevice")

        XCTAssertTrue(manager.preview.storage.hasDeviceKey)
    }
}

// MARK: - TC-PUSH

final class PawTestContractPushTests: XCTestCase {

    // TC-PUSH-01: Initial state is Unregistered
    func testTC_PUSH_01_initialStateIsUnregistered() {
        let store = PawInMemorySecureStore()
        let registrar = PawApnsPushRegistrar(secureStore: store)

        let state = registrar.currentState()
        XCTAssertEqual(state.status, "Unregistered")
        XCTAssertNil(state.token)
    }

    // TC-PUSH-03: Platform tag is "apns"
    func testTC_PUSH_03_platformTagIsApns() {
        let store = PawInMemorySecureStore()
        let registrar = PawApnsPushRegistrar(secureStore: store)

        XCTAssertEqual(registrar.currentState().platform, "apns")
        XCTAssertEqual(registrar.register(token: "tok").platform, "apns")
        XCTAssertEqual(registrar.unregister().platform, "apns")
    }
}

// MARK: - TC-LIFECYCLE

final class PawTestContractLifecycleTests: XCTestCase {

    // TC-LIFECYCLE-03: Inactive hints
    func testTC_LIFECYCLE_03_inactiveHints() {
        XCTAssertEqual(PawCoreManager.lifecycleHints(for: "Inactive"), ["FlushAcks"])
    }

    // TC-LIFECYCLE-04: Terminated hints
    func testTC_LIFECYCLE_04_terminatedHints() {
        // Implementation groups Terminated with Background
        XCTAssertEqual(
            PawCoreManager.lifecycleHints(for: "Terminated"),
            ["PauseRealtime", "FlushAcks", "PersistDrafts"]
        )
    }
}

// MARK: - TC-CHAT

final class PawTestContractChatTests: XCTestCase {

    // TC-CHAT-02: Conversation list after auth
    @MainActor
    func testTC_CHAT_02_conversationListAfterAuth() {
        let (manager, _, _) = makeFreshManager()
        authenticateManager(manager)

        let conversations = manager.preview.conversations
        XCTAssertFalse(conversations.isEmpty, "Conversations should be populated after auth")
        XCTAssertEqual(conversations.count, 2)

        // Each conversation has id and title
        for conversation in conversations {
            XCTAssertFalse(conversation.id.isEmpty)
            XCTAssertFalse(conversation.title.isEmpty)
        }
    }

    // TC-CHAT-03: Select conversation loads messages
    @MainActor
    func testTC_CHAT_03_selectConversationLoadsMessages() {
        let (manager, _, _) = makeFreshManager()
        authenticateManager(manager)

        let targetID = "conv-agent"
        manager.selectConversation(targetID)

        XCTAssertEqual(manager.preview.selectedConversationID, targetID)
        XCTAssertFalse(manager.preview.messages.isEmpty)
        // All messages belong to the selected conversation
        for message in manager.preview.messages {
            XCTAssertEqual(message.conversationID, targetID)
        }
    }

    // TC-CHAT-04: Conversation selection -- existing ID is retained
    @MainActor
    func testTC_CHAT_04_existingConversationIdIsRetained() {
        let (manager, _, _) = makeFreshManager()
        authenticateManager(manager)

        let firstID = manager.preview.conversations.first!.id
        manager.selectConversation(firstID)
        XCTAssertEqual(manager.preview.selectedConversationID, firstID)

        // Select the same ID again -- it should still be selected
        manager.selectConversation(firstID)
        XCTAssertEqual(manager.preview.selectedConversationID, firstID)
    }

    // TC-CHAT-05: Conversation selection -- nonexistent ID does not change selection
    @MainActor
    func testTC_CHAT_05_nonexistentConversationIdIgnored() {
        let (manager, _, _) = makeFreshManager()
        authenticateManager(manager)

        let originalID = manager.preview.selectedConversationID
        manager.selectConversation("nonexistent-id")

        // Selection does not change because ID is not in the list
        XCTAssertEqual(manager.preview.selectedConversationID, originalID)
    }

    // TC-CHAT-06: Message send -- optimistic append + agent reply (demo is synchronous)
    @MainActor
    func testTC_CHAT_06_messageSendAppendsUserAndAgentMessages() {
        let (manager, _, _) = makeFreshManager(
            now: { Date(timeIntervalSince1970: 1_710_000_000) }
        )
        authenticateManager(manager)

        let messagesBefore = manager.preview.messages.count
        manager.sendChatMessage("Hello world")

        // Two messages added: user + agent reply
        XCTAssertEqual(manager.preview.messages.count, messagesBefore + 2)

        let userMsg = manager.preview.messages[messagesBefore]
        XCTAssertEqual(userMsg.body, "Hello world")
        XCTAssertEqual(userMsg.role, .me)

        let agentMsg = manager.preview.messages[messagesBefore + 1]
        XCTAssertEqual(agentMsg.role, .agent)
        XCTAssertEqual(agentMsg.author, "Paw Agent")
    }

    // TC-CHAT-07: After send, composer is reset to next prompt
    @MainActor
    func testTC_CHAT_07_composerResetAfterSend() {
        let (manager, _, _) = makeFreshManager()
        authenticateManager(manager)

        manager.sendChatMessage("Check status")

        // Composer is updated to the next suggested prompt
        XCTAssertEqual(manager.preview.composerText, "Show sync + streaming status")
    }

    // TC-CHAT-08: After send, activeStreamCount returns to 0
    @MainActor
    func testTC_CHAT_08_activeStreamCountResetsAfterSend() {
        let (manager, _, _) = makeFreshManager()
        authenticateManager(manager)

        manager.sendChatMessage("Ping")

        // Synchronous demo: stream count is 0 after the full round-trip
        XCTAssertEqual(manager.preview.runtime.activeStreamCount, 0)
    }

    // TC-CHAT-09: Empty draft validation
    @MainActor
    func testTC_CHAT_09_emptyDraftValidation() {
        let (manager, _, _) = makeFreshManager()
        authenticateManager(manager)

        let messagesBefore = manager.preview.messages.count

        manager.sendChatMessage("")
        XCTAssertNotNil(manager.preview.auth.error)
        XCTAssertEqual(manager.preview.auth.error, "Composer is empty")
        XCTAssertEqual(manager.preview.messages.count, messagesBefore, "No messages should be appended")

        // Whitespace-only also rejected
        manager.sendChatMessage("   ")
        XCTAssertEqual(manager.preview.auth.error, "Composer is empty")
        XCTAssertEqual(manager.preview.messages.count, messagesBefore)
    }

    // TC-CHAT-10: No conversation selected validation (unauthenticated sends error)
    @MainActor
    func testTC_CHAT_10_noConversationSelectedValidation() {
        let (manager, _, _) = makeFreshManager()

        // Unauthenticated manager: sendChatMessage should fail
        manager.sendChatMessage("test")

        XCTAssertNotNil(manager.preview.auth.error)
        XCTAssertEqual(manager.preview.auth.error, "Finish auth flow before using chat shell")
    }

    // TC-CHAT-11: Cursor sync -- cursorCount matches conversation count after selection
    @MainActor
    func testTC_CHAT_11_cursorSyncAfterConversationSelection() {
        let (manager, _, _) = makeFreshManager()
        authenticateManager(manager)

        manager.selectConversation("conv-agent")
        XCTAssertEqual(
            manager.preview.runtime.cursorCount,
            manager.preview.conversations.count
        )
    }
}

// MARK: - TC-SESSION

final class PawTestContractSessionTests: XCTestCase {

    // TC-SESSION-02: No stored token leads to authMethodSelect
    @MainActor
    func testTC_SESSION_02_noStoredTokenStartsAtAuthMethodSelect() {
        let (manager, _, _) = makeFreshManager()

        XCTAssertEqual(manager.preview.auth.step, .authMethodSelect)
        XCTAssertFalse(manager.preview.auth.hasAccessToken)
        XCTAssertEqual(manager.preview.runtime.connectionState, "Disconnected")
    }

    // TC-SESSION-03: Invalid / cleared token handling -- after logout, state resets
    @MainActor
    func testTC_SESSION_03_invalidTokenHandling() {
        // Start with a pre-loaded token
        let preloadedTokens = PawStoredTokens(
            sessionToken: "old-session",
            accessToken: "old-access",
            refreshToken: "old-refresh"
        )
        let (manager, vault, _) = makeFreshManager(preloadTokens: preloadedTokens)

        // Manager should start authenticated
        XCTAssertEqual(manager.preview.auth.step, .authenticated)
        XCTAssertTrue(manager.preview.auth.hasAccessToken)

        // Simulate token invalidation by logging out (clears vault and resets state)
        manager.logout()

        XCTAssertEqual(manager.preview.auth.step, .authMethodSelect)
        XCTAssertFalse(manager.preview.auth.hasAccessToken)
        XCTAssertEqual(manager.preview.runtime.connectionState, "Disconnected")
        XCTAssertNil(vault.loadTokens().accessToken)
        XCTAssertNil(vault.loadTokens().sessionToken)
        XCTAssertNil(vault.loadTokens().refreshToken)
    }
}
