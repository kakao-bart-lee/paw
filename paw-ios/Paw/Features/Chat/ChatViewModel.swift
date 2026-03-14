import Foundation

/// Owns conversation list, messages, selected conversation, and chat actions.
/// All state is managed through the shared PawBootstrapPreview reference
/// to maintain backward compatibility with PawCoreManager's @Published preview.
@MainActor
final class ChatViewModel {
    private let now: () -> Date

    init(now: @escaping () -> Date = Date.init) {
        self.now = now
    }

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
