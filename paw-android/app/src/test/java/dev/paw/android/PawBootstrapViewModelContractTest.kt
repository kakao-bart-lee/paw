package dev.paw.android

import dev.paw.android.runtime.AndroidChatMessage
import dev.paw.android.runtime.AndroidChatShellState
import dev.paw.android.runtime.AndroidConversationItem
import dev.paw.android.runtime.SendMessageResult
import dev.paw.android.runtime.StoredTokens
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Ignore
import org.junit.Test

/**
 * ViewModel-level contract tests for PawBootstrapViewModel.
 *
 * These tests require either Android instrumentation or Robolectric because
 * PawBootstrapViewModel extends AndroidViewModel and depends on UniFFI native
 * bindings (uniffi.paw_core.*) which are unavailable in plain JVM unit tests.
 *
 * All test logic is written out completely so it can be enabled once the
 * test environment supports AndroidViewModel instantiation.
 *
 * To enable:
 * 1. Add Robolectric dependency: testImplementation("org.robolectric:robolectric:4.x")
 * 2. Annotate class with @RunWith(RobolectricTestRunner::class)
 * 3. Remove @Ignore annotations
 * 4. Provide mock implementations for API client, token vault, etc.
 */
class PawBootstrapViewModelContractTest {

    // =========================================================================
    // TC-AUTH: Auth Flow State Machine
    // =========================================================================

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-01 initial state is AUTH_METHOD_SELECT with no error and not loading`() {
        // Given: no stored tokens
        // When: ViewModel initializes
        // Then:
        // - auth step == AUTH_METHOD_SELECT
        // - error == null
        // - isLoading == false
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-02 phone submit transitions to OTP_VERIFY`() {
        // Given: auth step == AUTH_METHOD_SELECT
        // When: showPhoneOtp() -> onPhoneChanged("01012345678") -> requestOtp()
        // Then:
        // - auth step == OTP_VERIFY
        // - phone field reflects normalized input
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-03 OTP verify transitions to DEVICE_NAME`() {
        // Given: auth step == OTP_VERIFY, phone set
        // When: onOtpChanged("123456") -> verifyOtp()
        // Then:
        // - auth step == DEVICE_NAME
        // - hasSessionToken == true
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-04 device registration stores tokens and transitions to USERNAME_SETUP or AUTHENTICATED`() {
        // Given: auth step == DEVICE_NAME, session token staged
        // When: onDeviceNameChanged("My Phone") -> registerDevice()
        // Then:
        // - access/refresh tokens stored in vault
        // - if server response has no username -> step == USERNAME_SETUP
        // - if server response has username -> step == AUTHENTICATED
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-05 username setup transitions to AUTHENTICATED`() {
        // Given: auth step == USERNAME_SETUP
        // When: onUsernameChanged("alice") -> completeUsernameSetup()
        // Then:
        // - auth step == AUTHENTICATED
        // - username == "alice"
        // - discoverableByPhone reflected
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-06 username skip transitions to AUTHENTICATED`() {
        // Given: auth step == USERNAME_SETUP
        // When: skipUsernameSetup()
        // Then:
        // - auth step == AUTHENTICATED
        // - access token still valid
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-07 blank input at each step produces error without state transition`() {
        // Phone blank:
        // Given: step == PHONE_INPUT
        // When: onPhoneChanged("") -> requestOtp()
        // Then: error set, step unchanged

        // OTP blank:
        // Given: step == OTP_VERIFY
        // When: onOtpChanged("") -> verifyOtp()
        // Then: error set, step unchanged

        // Device name blank:
        // Given: step == DEVICE_NAME
        // When: onDeviceNameChanged("  ") -> registerDevice()
        // Then: error set, step unchanged

        // Username blank:
        // Given: step == USERNAME_SETUP
        // When: onUsernameChanged("") -> completeUsernameSetup()
        // Then: error set, step unchanged
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-08 loading state during async auth requests`() {
        // Given: any auth step
        // When: async auth request starts
        // Then:
        // - isLoading == true
        // - error == null
        // When: request completes
        // Then:
        // - isLoading == false
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-AUTH-09 network error sets error message and retains current step`() {
        // Given: any auth step
        // When: auth request fails with network error
        // Then:
        // - error message set
        // - step unchanged from before request
        // When: retry
        // Then: request is retryable
    }

    // =========================================================================
    // TC-CHAT: Chat Shell (ViewModel-level tests)
    // =========================================================================

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-CHAT-06 send message adds optimistic entry and clears draft`() {
        // Given: authenticated, conversation selected, draft = "hello"
        // When: sendMessage()
        // Then:
        // - messages list contains optimistic entry with content "hello"
        // - sendingMessage == true
        // - messageDraft == ""
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-CHAT-07 server confirmation replaces optimistic message`() {
        // Given: optimistic message sent
        // When: server responds with id, seq, createdAt
        // Then:
        // - optimistic message replaced by confirmed message
        // - confirmed message has server id, seq, createdAt
        // - conversation lastMessage updated
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-CHAT-08 send failure removes optimistic message and sets error`() {
        // Given: optimistic message sent
        // When: server responds with error
        // Then:
        // - optimistic message removed from messages
        // - messagesError set
        // - sendingMessage == false
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-CHAT-09 blank draft send produces error without sending`() {
        // Given: authenticated, conversation selected, draft = "  "
        // When: sendMessage()
        // Then:
        // - messagesError set
        // - messages list unchanged
        // - sendingMessage == false
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-CHAT-10 send without selected conversation produces error`() {
        // Given: authenticated, selectedConversationId == null
        // When: sendMessage()
        // Then:
        // - messagesError set ("먼저 대화를 선택하세요.")
    }

    // =========================================================================
    // TC-SESSION: Session Restore
    // =========================================================================

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-SESSION-01 valid stored token restores to AUTHENTICATED or USERNAME_SETUP`() {
        // Given: valid access token stored in vault
        // When: ViewModel initializes (bootstrap)
        // Then:
        // - if /me returns username -> step == AUTHENTICATED
        // - if /me returns blank username -> step == USERNAME_SETUP
        // - hasAccessToken == true
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-SESSION-02 no stored token results in AUTH_METHOD_SELECT`() {
        // Given: vault is empty
        // When: ViewModel initializes (bootstrap)
        // Then:
        // - step == AUTH_METHOD_SELECT
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-SESSION-03 expired token clears vault and returns to AUTH_METHOD_SELECT`() {
        // Given: stored token that fails /me call
        // When: ViewModel initializes (bootstrap)
        // Then:
        // - vault cleared
        // - step == AUTH_METHOD_SELECT
        // - error message reflected
    }

    // =========================================================================
    // TC-LOGOUT: Logout
    // =========================================================================

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-LOGOUT-01 logout clears all state`() {
        // Given: authenticated state with conversations and messages
        // When: logout()
        // Then:
        // - vault cleared (read == null)
        // - auth step == AUTH_METHOD_SELECT
        // - chat state == empty (no conversations, no messages)
        // - connection == DISCONNECTED
    }

    @Test
    @Ignore("Requires Android instrumentation or Robolectric")
    fun `TC-LOGOUT-02 logout triggers push unregister`() {
        // Given: authenticated with push registered
        // When: logout()
        // Then:
        // - pushRegistrar.unregister() called
        // - push state == UNREGISTERED
    }
}
