import Foundation
import Security

private enum PawSecureStoreKey {
    static let sessionToken = "dev.paw.auth.session"
    static let accessToken = "dev.paw.auth.access"
    static let refreshToken = "dev.paw.auth.refresh"
    static let deviceKey = "dev.paw.device.key"
    static let pushToken = "dev.paw.push.token"
}

final class PawKeychainStore: PawKeyValueSecureStore {
    func data(forKey key: String) -> Data? {
        var item: CFTypeRef?
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitOne,
        ]
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        guard status == errSecSuccess else {
            return nil
        }
        return item as? Data
    }

    @discardableResult
    func set(_ data: Data, forKey key: String) -> Bool {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
        ]
        let attributes: [CFString: Any] = [kSecValueData: data]
        let updateStatus = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        if updateStatus == errSecSuccess {
            return true
        }

        var insert = query
        insert[kSecValueData] = data
        insert[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        return SecItemAdd(insert as CFDictionary, nil) == errSecSuccess
    }

    @discardableResult
    func removeValue(forKey key: String) -> Bool {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
        ]
        let status = SecItemDelete(query as CFDictionary)
        return status == errSecSuccess || status == errSecItemNotFound
    }
}

final class PawKeychainTokenVault: PawTokenVault {
    private let secureStore: PawKeyValueSecureStore

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore()) {
        self.secureStore = secureStore
    }

    func loadTokens() -> PawStoredTokens {
        PawStoredTokens(
            sessionToken: decode(PawSecureStoreKey.sessionToken),
            accessToken: decode(PawSecureStoreKey.accessToken),
            refreshToken: decode(PawSecureStoreKey.refreshToken)
        )
    }

    func save(tokens: PawStoredTokens) -> Bool {
        persist(tokens.sessionToken, forKey: PawSecureStoreKey.sessionToken)
            && persist(tokens.accessToken, forKey: PawSecureStoreKey.accessToken)
            && persist(tokens.refreshToken, forKey: PawSecureStoreKey.refreshToken)
    }

    func clearTokens() -> Bool {
        secureStore.removeValue(forKey: PawSecureStoreKey.sessionToken)
            && secureStore.removeValue(forKey: PawSecureStoreKey.accessToken)
            && secureStore.removeValue(forKey: PawSecureStoreKey.refreshToken)
    }

    func storagePreview(hasDeviceKey: Bool) -> PawStoragePreview {
        PawStoragePreview(
            provider: "Keychain",
            availability: "Available",
            hasDeviceKey: hasDeviceKey
        )
    }

    private func decode(_ key: String) -> String? {
        guard let data = secureStore.data(forKey: key) else {
            return nil
        }
        return String(data: data, encoding: .utf8)
    }

    private func persist(_ value: String?, forKey key: String) -> Bool {
        guard let value else {
            return secureStore.removeValue(forKey: key)
        }
        return secureStore.set(Data(value.utf8), forKey: key)
    }
}

final class PawKeychainDeviceKeyStore: PawDeviceKeyStore {
    private let secureStore: PawKeyValueSecureStore

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore()) {
        self.secureStore = secureStore
    }

    func hasDeviceKey() -> Bool {
        secureStore.data(forKey: PawSecureStoreKey.deviceKey) != nil
    }

    func saveDeviceKey(_ data: Data) -> Bool {
        secureStore.set(data, forKey: PawSecureStoreKey.deviceKey)
    }

    func clearDeviceKey() -> Bool {
        secureStore.removeValue(forKey: PawSecureStoreKey.deviceKey)
    }
}

final class PawApnsPushRegistrar: PawPushRegistrar {
    private let secureStore: PawKeyValueSecureStore
    private let now: () -> Date
    private var lastError: String?

    init(secureStore: PawKeyValueSecureStore = PawKeychainStore(), now: @escaping () -> Date = Date.init) {
        self.secureStore = secureStore
        self.now = now
    }

    func currentState() -> PawPushPreview {
        let token = secureStore.data(forKey: PawSecureStoreKey.pushToken)
            .flatMap { String(data: $0, encoding: .utf8) }
        return PawPushPreview(
            status: token == nil ? "Unregistered" : "Registered",
            platform: "apns",
            token: token,
            lastError: lastError,
            lastUpdatedMs: Int(now().timeIntervalSince1970 * 1_000)
        )
    }

    func register(token: String) -> PawPushPreview {
        guard !token.isEmpty else {
            lastError = "Empty APNs token"
            return currentState()
        }
        _ = secureStore.set(Data(token.utf8), forKey: PawSecureStoreKey.pushToken)
        lastError = nil
        return currentState()
    }

    func unregister() -> PawPushPreview {
        _ = secureStore.removeValue(forKey: PawSecureStoreKey.pushToken)
        lastError = nil
        return currentState()
    }
}

@MainActor
final class PawCoreManager: ObservableObject {
    @Published private(set) var bindingsStatus = "Native iOS bootstrap ready"
    @Published private(set) var preview: PawBootstrapPreview

    private let tokenVault: PawTokenVault
    private let deviceKeyStore: PawDeviceKeyStore
    private let pushRegistrar: PawPushRegistrar
    private let now: () -> Date

    init(
        tokenVault: PawTokenVault = PawKeychainTokenVault(),
        deviceKeyStore: PawDeviceKeyStore = PawKeychainDeviceKeyStore(),
        pushRegistrar: PawPushRegistrar = PawApnsPushRegistrar(),
        now: @escaping () -> Date = Date.init
    ) {
        self.tokenVault = tokenVault
        self.deviceKeyStore = deviceKeyStore
        self.pushRegistrar = pushRegistrar
        self.now = now

        let hasDeviceKey = deviceKeyStore.hasDeviceKey()
        let tokens = tokenVault.loadTokens()
        let shell = Self.makeShellState(authenticated: tokens.accessToken != nil, username: nil)
        self.preview = PawBootstrapPreview(
            bridgeStatus: "Keychain + APNs adapters wired",
            auth: PawCoreManager.makeAuthPreview(tokens: tokens),
            runtime: PawRuntimePreview(
                connectionState: tokens.accessToken == nil ? "Disconnected" : "Ready",
                cursorCount: shell.conversations.count,
                activeStreamCount: 0
            ),
            storage: tokenVault.storagePreview(hasDeviceKey: hasDeviceKey),
            push: pushRegistrar.currentState(),
            lifecycle: PawLifecyclePreview(
                activeHints: Self.lifecycleHints(for: "Active"),
                backgroundHints: Self.lifecycleHints(for: "Background"),
                currentState: "Launching"
            ),
            conversations: shell.conversations,
            selectedConversationID: shell.selectedConversationID,
            messages: shell.messages,
            composerText: shell.composerText,
            shellBanner: shell.banner
        )
    }

    var artifactsDirectory: String {
        "PawCore/Artifacts"
    }

    var selectedConversation: PawConversationPreview? {
        guard let id = preview.selectedConversationID else { return nil }
        return preview.conversations.first(where: { $0.id == id })
    }

    func startPhoneInput() {
        preview.auth.step = .phoneInput
        preview.auth.error = nil
        updateShellBannerForCurrentState()
    }

    func submitPhone(_ phone: String = "+82 10-5555-0101", discoverableByPhone: Bool = true) {
        preview.auth.phone = phone
        preview.auth.discoverableByPhone = discoverableByPhone
        preview.auth.step = .otpVerify
        preview.auth.error = nil
        updateShellBannerForCurrentState()
    }

    func verifyOtp(_ code: String = "123456") {
        guard code.count >= 6 else {
            preview.auth.error = "OTP must be 6 digits"
            return
        }

        let tokens = PawStoredTokens(
            sessionToken: "session-ios-demo",
            accessToken: "access-ios-demo",
            refreshToken: "refresh-ios-demo"
        )
        guard tokenVault.save(tokens: tokens) else {
            preview.auth.error = "Failed to persist session in Keychain"
            return
        }

        preview.auth.hasSessionToken = true
        preview.auth.hasAccessToken = true
        preview.auth.hasRefreshToken = true
        preview.auth.step = .deviceName
        preview.runtime.connectionState = "Bootstrapping"
        preview.auth.error = nil
        updateShellBannerForCurrentState()
    }

    func submitDeviceName(_ deviceName: String = "Haruna's iPhone") {
        preview.auth.deviceName = deviceName
        preview.auth.step = .usernameSetup
        preview.auth.error = nil
        if !deviceKeyStore.hasDeviceKey() {
            _ = deviceKeyStore.saveDeviceKey(Data(deviceName.utf8))
            preview.storage.hasDeviceKey = deviceKeyStore.hasDeviceKey()
        }
        updateShellBannerForCurrentState()
    }

    func submitUsername(_ username: String = "haruna") {
        preview.auth.username = username
        preview.auth.step = .authenticated
        preview.runtime.connectionState = "Connected"
        preview.auth.error = nil
        hydrateChatShell(username: username)
    }

    func skipUsername() {
        preview.auth.step = .authenticated
        preview.runtime.connectionState = "Connected"
        preview.auth.error = nil
        hydrateChatShell(username: effectiveUsername)
    }

    func refresh() {
        updateShellBannerForCurrentState()
    }

    func logout() {
        _ = tokenVault.clearTokens()
        _ = pushRegistrar.unregister()
        preview.auth = Self.makeAuthPreview(tokens: PawStoredTokens(sessionToken: nil, accessToken: nil, refreshToken: nil))
        preview.runtime.connectionState = "Disconnected"
        preview.push = pushRegistrar.currentState()
        hydrateChatShell(username: nil)
    }

    func registerForPush(token: String = "apns-demo-token") {
        preview.push = pushRegistrar.register(token: token)
        updateShellBannerForCurrentState()
    }

    func unregisterPush() {
        preview.push = pushRegistrar.unregister()
        updateShellBannerForCurrentState()
    }

    func applyLifecycle(state: String) {
        preview.lifecycle.currentState = state
        switch state {
        case "Active":
            preview.lifecycle.activeHints = Self.lifecycleHints(for: state)
            preview.runtime.connectionState = preview.auth.hasAccessToken ? "Connected" : "Disconnected"
        case "Background":
            preview.lifecycle.backgroundHints = Self.lifecycleHints(for: state)
            preview.runtime.connectionState = "Background"
            preview.runtime.activeStreamCount = 0
        default:
            break
        }
        updateShellBannerForCurrentState()
    }

    func selectConversation(_ id: String) {
        guard preview.auth.hasAccessToken else {
            preview.auth.error = "Authenticate first to open conversations"
            updateShellBannerForCurrentState()
            return
        }
        guard preview.conversations.contains(where: { $0.id == id }) else {
            return
        }
        preview.selectedConversationID = id
        preview.messages = Self.baseMessages(for: id, username: effectiveUsername)
        updateConversationMetadata(for: id, unreadCount: 0)
        preview.runtime.cursorCount = preview.conversations.count
        updateShellBannerForCurrentState()
    }

    func selectNextConversation() {
        guard let selectedConversationID else {
            selectConversation(preview.conversations.first?.id ?? "")
            return
        }
        guard let index = preview.conversations.firstIndex(where: { $0.id == selectedConversationID }) else {
            return
        }
        let next = preview.conversations[(index + 1) % preview.conversations.count]
        selectConversation(next.id)
    }

    func sendChatMessage(_ text: String? = nil) {
        guard preview.auth.hasAccessToken, let conversationID = preview.selectedConversationID else {
            preview.auth.error = "Finish auth flow before using chat shell"
            updateShellBannerForCurrentState()
            return
        }

        let outgoing = (text ?? preview.composerText).trimmingCharacters(in: .whitespacesAndNewlines)
        guard !outgoing.isEmpty else {
            preview.auth.error = "Composer is empty"
            return
        }

        preview.auth.error = nil
        preview.messages.append(
            PawMessagePreview(
                id: "msg-\(preview.messages.count + 1)-me",
                conversationID: conversationID,
                author: effectiveUsername,
                body: outgoing,
                role: .me,
                timestampLabel: Self.timestampLabel(from: now())
            )
        )
        preview.runtime.activeStreamCount = 1
        preview.runtime.connectionState = preview.lifecycle.currentState == "Background" ? "Background" : "Connected"

        let reply = agentReply(for: outgoing)
        preview.messages.append(
            PawMessagePreview(
                id: "msg-\(preview.messages.count + 1)-agent",
                conversationID: conversationID,
                author: "Paw Agent",
                body: reply,
                role: .agent,
                timestampLabel: Self.timestampLabel(from: now())
            )
        )
        preview.runtime.activeStreamCount = 0
        preview.composerText = "Show sync + streaming status"
        updateConversationMetadata(for: conversationID, subtitle: reply, unreadCount: 0)
        updateShellBannerForCurrentState()
    }

    private var selectedConversationID: String? {
        preview.selectedConversationID
    }

    private var effectiveUsername: String {
        preview.auth.username.isEmpty ? "haruna" : preview.auth.username
    }

    private func hydrateChatShell(username: String?) {
        let shell = Self.makeShellState(authenticated: preview.auth.hasAccessToken, username: username ?? Self.nilIfEmpty(preview.auth.username))
        preview.conversations = shell.conversations
        preview.selectedConversationID = shell.selectedConversationID
        preview.messages = shell.messages
        preview.composerText = shell.composerText
        preview.shellBanner = shell.banner
        preview.runtime.cursorCount = shell.conversations.count
        preview.runtime.activeStreamCount = 0
    }

    private func updateConversationMetadata(for conversationID: String, subtitle: String? = nil, unreadCount: Int? = nil) {
        preview.conversations = preview.conversations.map { conversation in
            guard conversation.id == conversationID else { return conversation }
            return PawConversationPreview(
                id: conversation.id,
                title: conversation.title,
                subtitle: subtitle ?? conversation.subtitle,
                unreadCount: unreadCount ?? conversation.unreadCount,
                accent: conversation.accent
            )
        }
    }

    private func agentReply(for outgoing: String) -> String {
        let pushState = preview.push.status == "Registered" ? "APNs active" : "APNs pending"
        let lifecycle = preview.lifecycle.currentState
        return "Runtime live: \(preview.runtime.connectionState), \(pushState), lifecycle=\(lifecycle). Echoing \"\(outgoing)\" into the iOS shell."
    }

    private func updateShellBannerForCurrentState() {
        if !preview.auth.hasAccessToken {
            preview.shellBanner = "Authenticate to unlock conversations + chat runtime shell."
            return
        }

        let conversation = selectedConversation?.title ?? "No conversation"
        preview.shellBanner = "\(conversation) · \(preview.runtime.connectionState) · push \(preview.push.status.lowercased())"
    }

    private static func makeAuthPreview(tokens: PawStoredTokens) -> PawAuthPreview {
        PawAuthPreview(
            step: tokens.accessToken == nil ? .authMethodSelect : .authenticated,
            phone: "",
            deviceName: "",
            username: "",
            discoverableByPhone: false,
            hasSessionToken: tokens.sessionToken != nil,
            hasAccessToken: tokens.accessToken != nil,
            hasRefreshToken: tokens.refreshToken != nil,
            isLoading: false,
            error: nil
        )
    }

    private static func makeShellState(authenticated: Bool, username: String?) -> (conversations: [PawConversationPreview], selectedConversationID: String?, messages: [PawMessagePreview], composerText: String, banner: String) {
        let resolvedUsername = nilIfEmpty(username) ?? "haruna"
        let conversations = [
            PawConversationPreview(id: "conv-bootstrap", title: "Bootstrap Crew", subtitle: authenticated ? "Runtime snapshot restored from Keychain." : "Sign in to hydrate bootstrap thread.", unreadCount: authenticated ? 0 : 2, accent: "primary"),
            PawConversationPreview(id: "conv-agent", title: "Agent Ops", subtitle: authenticated ? "Streaming shell ready for agent replies." : "Agent shell locked until auth completes.", unreadCount: authenticated ? 1 : 0, accent: "accent")
        ]
        let selectedConversationID = conversations.first?.id
        let messages = baseMessages(for: selectedConversationID ?? "conv-bootstrap", username: resolvedUsername, authenticated: authenticated)
        let composerText = authenticated ? "Summarize bootstrap status" : "Authenticate to enable chat"
        let banner = authenticated ? "\(conversations[0].title) · Ready · push pending" : "Authenticate to unlock conversations + chat runtime shell."
        return (conversations, selectedConversationID, messages, composerText, banner)
    }

    private static func baseMessages(for conversationID: String, username: String, authenticated: Bool = true) -> [PawMessagePreview] {
        switch conversationID {
        case "conv-agent":
            return [
                PawMessagePreview(id: "agent-1", conversationID: conversationID, author: "Paw Agent", body: authenticated ? "Streaming slot is warm. Ask for runtime state when ready." : "Finish auth to open the agent runtime shell.", role: .agent, timestampLabel: "now"),
                PawMessagePreview(id: "agent-2", conversationID: conversationID, author: username, body: authenticated ? "Show me the iOS bootstrap summary." : "Waiting for OTP verification.", role: .me, timestampLabel: "now")
            ]
        default:
            return [
                PawMessagePreview(id: "bootstrap-1", conversationID: conversationID, author: "System", body: authenticated ? "Stored access token restored from Keychain and runtime snapshot is live." : "No stored token found. Auth flow starts from phone input.", role: .peer, timestampLabel: "now"),
                PawMessagePreview(id: "bootstrap-2", conversationID: conversationID, author: "Paw Agent", body: authenticated ? "APNs + lifecycle hints are bound to the shell cards below." : "Complete auth to hydrate conversations and chat runtime.", role: .agent, timestampLabel: "now")
            ]
        }
    }

    private static func timestampLabel(from date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: date)
    }

    nonisolated static func lifecycleHints(for state: String) -> [String] {
        switch state {
        case "Launching", "Active":
            ["ReconnectSocket", "RefreshPushToken"]
        case "Inactive":
            ["FlushAcks"]
        case "Background", "Terminated":
            ["PauseRealtime", "FlushAcks", "PersistDrafts"]
        default:
            []
        }
    }
}

private extension PawCoreManager {
    static func nilIfEmpty(_ value: String?) -> String? {
        guard let value, !value.isEmpty else { return nil }
        return value
    }
}
