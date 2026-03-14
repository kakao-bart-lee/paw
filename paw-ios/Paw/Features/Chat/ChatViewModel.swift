import Foundation

/// Owns conversation list, messages, selected conversation, and chat actions.
/// All state is managed through the shared PawBootstrapPreview reference
/// to maintain backward compatibility with PawCoreManager's @Published preview.
@MainActor
final class ChatViewModel {
    private let now: () -> Date
    private let apiClient: PawApiClient

    init(
        now: @escaping () -> Date = Date.init,
        apiClient: PawApiClient = PawApiClient()
    ) {
        self.now = now
        self.apiClient = apiClient
    }

    // MARK: - API-backed operations (used by views)

    func hydrateChatFromAPI(
        authenticated: Bool,
        username: String?,
        preview: inout PawBootstrapPreview
    ) async {
        guard authenticated else {
            preview.conversations = []
            preview.selectedConversationID = nil
            preview.messages = []
            preview.composerText = "Authenticate to enable chat"
            preview.shellBanner = "Authenticate to unlock conversations + chat runtime shell."
            return
        }

        do {
            let rawConversations = try await apiClient.getConversations()
            let conversations = rawConversations.enumerated().map { index, raw in
                PawConversationPreview(
                    id: raw["id"] as? String ?? "conv-\(index)",
                    title: (raw["name"] as? String).flatMap { $0.isEmpty ? nil : $0 }
                        ?? "Conversation \(index + 1)",
                    subtitle: (raw["last_message"] as? String) ?? "",
                    unreadCount: raw["unread_count"] as? Int ?? 0,
                    accent: index == 0 ? "primary" : "accent"
                )
            }

            preview.conversations = conversations
            preview.selectedConversationID = conversations.first?.id

            if let firstId = conversations.first?.id {
                let rawMessages = try await apiClient.getMessages(conversationId: firstId)
                let resolvedUsername = Self.nilIfEmpty(username) ?? "me"
                preview.messages = rawMessages.enumerated().map { index, raw in
                    let senderId = raw["sender_id"] as? String ?? "unknown"
                    let role: PawMessagePreview.Role = senderId == resolvedUsername ? .me
                        : (raw["format"] as? String == "agent" ? .agent : .peer)
                    return PawMessagePreview(
                        id: raw["id"] as? String ?? "msg-\(index)",
                        conversationID: firstId,
                        author: senderId,
                        body: raw["content"] as? String ?? "",
                        role: role,
                        timestampLabel: (raw["created_at"] as? String) ?? Self.timestampLabel(from: Date())
                    )
                }
            } else {
                preview.messages = []
            }

            preview.composerText = conversations.isEmpty
                ? "No conversations yet"
                : "Summarize bootstrap status"
            preview.shellBanner = conversations.isEmpty
                ? "No conversations available."
                : "\(conversations.first?.title ?? "Chat") \u{00B7} Ready"
            preview.runtime.cursorCount = conversations.count
            preview.runtime.activeStreamCount = 0
        } catch {
            // API failed: show empty state rather than hardcoded data
            preview.conversations = []
            preview.selectedConversationID = nil
            preview.messages = []
            preview.composerText = "Failed to load conversations"
            preview.shellBanner = "Could not reach server."
            preview.auth.error = error.localizedDescription
        }
    }

    func selectConversationAsync(
        _ id: String,
        preview: inout PawBootstrapPreview,
        effectiveUsername: String
    ) async {
        guard preview.auth.hasAccessToken else {
            preview.auth.error = "Authenticate first to open conversations"
            return
        }
        guard preview.conversations.contains(where: { $0.id == id }) else {
            return
        }

        preview.selectedConversationID = id
        preview.auth.isLoading = true

        do {
            let rawMessages = try await apiClient.getMessages(conversationId: id)
            preview.messages = rawMessages.enumerated().map { index, raw in
                let senderId = raw["sender_id"] as? String ?? "unknown"
                let role: PawMessagePreview.Role = senderId == effectiveUsername ? .me
                    : (raw["format"] as? String == "agent" ? .agent : .peer)
                return PawMessagePreview(
                    id: raw["id"] as? String ?? "msg-\(index)",
                    conversationID: id,
                    author: senderId,
                    body: raw["content"] as? String ?? "",
                    role: role,
                    timestampLabel: (raw["created_at"] as? String) ?? Self.timestampLabel(from: Date())
                )
            }
        } catch {
            preview.messages = []
            preview.auth.error = error.localizedDescription
        }

        preview.conversations = updateConversationMetadata(
            in: preview.conversations,
            for: id,
            unreadCount: 0
        )
        preview.runtime.cursorCount = preview.conversations.count
        preview.auth.isLoading = false
    }

    func selectNextConversationAsync(
        preview: inout PawBootstrapPreview,
        effectiveUsername: String
    ) async {
        guard let selectedID = preview.selectedConversationID else {
            if let firstID = preview.conversations.first?.id {
                await selectConversationAsync(firstID, preview: &preview, effectiveUsername: effectiveUsername)
            }
            return
        }
        guard let index = preview.conversations.firstIndex(where: { $0.id == selectedID }) else {
            return
        }
        let next = preview.conversations[(index + 1) % preview.conversations.count]
        await selectConversationAsync(next.id, preview: &preview, effectiveUsername: effectiveUsername)
    }

    func sendChatMessageToAPI(
        _ text: String?,
        preview: inout PawBootstrapPreview
    ) async {
        guard preview.auth.hasAccessToken, let conversationID = preview.selectedConversationID else {
            preview.auth.error = "Finish auth flow before using chat"
            return
        }

        let outgoing = (text ?? preview.composerText).trimmingCharacters(in: .whitespacesAndNewlines)
        guard !outgoing.isEmpty else {
            preview.auth.error = "Message cannot be empty"
            return
        }

        preview.auth.error = nil
        preview.runtime.activeStreamCount = 1

        do {
            let result = try await apiClient.sendMessage(
                conversationId: conversationID,
                content: outgoing
            )

            let msgId = result["id"] as? String ?? "msg-\(preview.messages.count + 1)"
            let createdAt = result["created_at"] as? String ?? Self.timestampLabel(from: now())

            preview.messages.append(
                PawMessagePreview(
                    id: msgId,
                    conversationID: conversationID,
                    author: "me",
                    body: outgoing,
                    role: .me,
                    timestampLabel: createdAt
                )
            )

            preview.composerText = ""
            preview.conversations = updateConversationMetadata(
                in: preview.conversations,
                for: conversationID,
                subtitle: outgoing,
                unreadCount: 0
            )
        } catch {
            preview.auth.error = error.localizedDescription
        }

        preview.runtime.activeStreamCount = 0
    }

    // MARK: - Synchronous methods (backward-compat for tests)

    func selectConversation(
        _ id: String,
        preview: inout PawBootstrapPreview,
        effectiveUsername: String
    ) {
        guard preview.auth.hasAccessToken else {
            preview.auth.error = "Authenticate first to open conversations"
            return
        }
        guard preview.conversations.contains(where: { $0.id == id }) else {
            return
        }
        preview.selectedConversationID = id
        preview.messages = Self.baseMessages(for: id, username: effectiveUsername)
        preview.conversations = updateConversationMetadata(
            in: preview.conversations,
            for: id,
            unreadCount: 0
        )
        preview.runtime.cursorCount = preview.conversations.count
    }

    func selectNextConversation(
        preview: inout PawBootstrapPreview,
        effectiveUsername: String
    ) {
        guard let selectedID = preview.selectedConversationID else {
            if let firstID = preview.conversations.first?.id {
                selectConversation(firstID, preview: &preview, effectiveUsername: effectiveUsername)
            }
            return
        }
        guard let index = preview.conversations.firstIndex(where: { $0.id == selectedID }) else {
            return
        }
        let next = preview.conversations[(index + 1) % preview.conversations.count]
        selectConversation(next.id, preview: &preview, effectiveUsername: effectiveUsername)
    }

    func sendChatMessage(
        _ text: String? = nil,
        preview: inout PawBootstrapPreview,
        effectiveUsername: String,
        agentReplyBuilder: (String) -> String
    ) {
        guard preview.auth.hasAccessToken, let conversationID = preview.selectedConversationID else {
            preview.auth.error = "Finish auth flow before using chat shell"
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

        let reply = agentReplyBuilder(outgoing)
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
        preview.conversations = updateConversationMetadata(
            in: preview.conversations,
            for: conversationID,
            subtitle: reply,
            unreadCount: 0
        )
    }

    func hydrateChatShell(
        authenticated: Bool,
        username: String?,
        preview: inout PawBootstrapPreview
    ) {
        let shell = Self.makeShellState(authenticated: authenticated, username: username)
        preview.conversations = shell.conversations
        preview.selectedConversationID = shell.selectedConversationID
        preview.messages = shell.messages
        preview.composerText = shell.composerText
        preview.shellBanner = shell.banner
        preview.runtime.cursorCount = shell.conversations.count
        preview.runtime.activeStreamCount = 0
    }

    // MARK: - Static helpers

    static func makeShellState(
        authenticated: Bool,
        username: String?
    ) -> (conversations: [PawConversationPreview], selectedConversationID: String?, messages: [PawMessagePreview], composerText: String, banner: String) {
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

    static func baseMessages(
        for conversationID: String,
        username: String,
        authenticated: Bool = true
    ) -> [PawMessagePreview] {
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

    static func timestampLabel(from date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: date)
    }

    // MARK: - Private helpers

    private func updateConversationMetadata(
        in conversations: [PawConversationPreview],
        for conversationID: String,
        subtitle: String? = nil,
        unreadCount: Int? = nil
    ) -> [PawConversationPreview] {
        conversations.map { conversation in
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

    private static func nilIfEmpty(_ value: String?) -> String? {
        guard let value, !value.isEmpty else { return nil }
        return value
    }
}
