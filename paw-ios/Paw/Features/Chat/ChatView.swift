import SwiftUI

struct ChatView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            conversationListSection
            chatWorkspace
        }
    }

    // MARK: - Conversation list

    private var conversationListSection: some View {
        shellCard(
            title: "Conversations",
            subtitle: "recent threads, active selection, and handoff-ready context",
            background: PawTheme.surface1
        ) {
            if coreManager.preview.conversations.isEmpty {
                emptyState(
                    title: "No conversations yet",
                    detail: "The runtime is ready, but no threads have been hydrated into the local shell.",
                    identifier: PawAccessibility.conversationsEmpty
                )
            } else {
                VStack(alignment: .leading, spacing: 10) {
                    Text(coreManager.selectedConversation?.title ?? "No active thread")
                        .font(PawTypography.headlineMedium)
                        .foregroundStyle(PawTheme.strongText)
                    Text(coreManager.selectedConversation?.subtitle ?? "Pick a conversation to restore detail and composer context.")
                        .font(PawTypography.bodySmall)
                        .foregroundStyle(PawTheme.mutedText)

                    VStack(alignment: .leading, spacing: 10) {
                        ForEach(coreManager.preview.conversations) { conversation in
                            conversationRow(conversation, selected: coreManager.preview.selectedConversationID == conversation.id)
                        }
                    }
                }
            }
        }
    }

    // MARK: - Chat workspace

    private var chatWorkspace: some View {
        shellCard(
            title: "Chat runtime",
            subtitle: "thread detail, composer prompt, and runtime actions",
            background: PawTheme.surface3
        ) {
            if coreManager.preview.messages.isEmpty {
                emptyState(
                    title: "No messages in this thread",
                    detail: "Pick another conversation or send the first runtime prompt from this restored workspace.",
                    identifier: PawAccessibility.chatEmpty
                )
            } else {
                VStack(alignment: .leading, spacing: 10) {
                    Text(coreManager.preview.composerText)
                        .font(PawTypography.titleMedium)
                        .foregroundStyle(PawTheme.strongText)
                        .accessibilityIdentifier(PawAccessibility.composer)
                    Text(chatSummaryCopy)
                        .font(PawTypography.bodySmall)
                        .foregroundStyle(PawTheme.mutedText)
                    VStack(alignment: .leading, spacing: 8) {
                        ForEach(coreManager.preview.messages) { message in
                            messageBubble(message)
                        }
                    }
                    .accessibilityIdentifier(PawAccessibility.messageList)
                    HStack(spacing: 8) {
                        authChip(title: "질문 보내기", selected: false, identifier: PawAccessibility.sendMessageButton) {
                            coreManager.sendChatMessage()
                        }
                        authChip(title: "다음 대화", selected: false, identifier: PawAccessibility.nextConversationButton) {
                            coreManager.selectNextConversation()
                        }
                    }
                }
            }
        }
    }

    private var chatSummaryCopy: String {
        "\(coreManager.preview.messages.count) messages · \(coreManager.preview.runtime.activeStreamCount) active streams · selected thread \(coreManager.selectedConversation?.title ?? "none")"
    }

    // MARK: - Conversation row

    @ViewBuilder
    private func conversationRow(_ conversation: PawConversationPreview, selected: Bool) -> some View {
        Button {
            coreManager.selectConversation(conversation.id)
        } label: {
            HStack(alignment: .top, spacing: 10) {
                Circle()
                    .fill(accentColor(conversation.accent))
                    .frame(width: 10, height: 10)
                    .padding(.top, 5)
                VStack(alignment: .leading, spacing: 4) {
                    Text(conversation.title)
                        .font(PawTypography.bodyMedium)
                        .foregroundStyle(PawTheme.strongText)
                    Text(conversation.subtitle)
                        .font(PawTypography.bodySmall)
                        .foregroundStyle(PawTheme.mutedText)
                }
                Spacer(minLength: 12)
                if conversation.unreadCount > 0 {
                    Text("\(conversation.unreadCount)")
                        .font(PawTypography.labelSmall)
                        .foregroundStyle(PawTheme.background)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(PawTheme.accent)
                        .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))
                }
            }
            .padding(12)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(selected ? PawTheme.surface4 : PawTheme.surface2)
            .overlay(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .stroke(selected ? PawTheme.accent : PawTheme.outline, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
        }
        .buttonStyle(.plain)
    }

    // MARK: - Message bubble

    @ViewBuilder
    private func messageBubble(_ message: PawMessagePreview) -> some View {
        HStack {
            if message.role == .me {
                Spacer(minLength: 24)
            }
            VStack(alignment: .leading, spacing: 4) {
                Text(message.author)
                    .font(PawTypography.labelSmall)
                    .foregroundStyle(PawTheme.accent)
                Text(message.body)
                    .font(PawTypography.bodyMedium)
                    .foregroundStyle(PawTheme.strongText)
                Text(message.timestampLabel)
                    .font(PawTypography.bodySmall)
                    .foregroundStyle(PawTheme.mutedText)
            }
            .padding(12)
            .background(bubbleColor(for: message.role))
            .overlay(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .stroke(PawTheme.outline, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
            if message.role != .me {
                Spacer(minLength: 24)
            }
        }
    }

    private func bubbleColor(for role: PawMessagePreview.Role) -> Color {
        switch role {
        case .me:
            PawTheme.sentBubble
        case .peer:
            PawTheme.receivedBubble
        case .agent:
            PawTheme.agentBubble
        }
    }

    private func accentColor(_ accent: String) -> Color {
        switch accent {
        case "primary", "accent":
            PawTheme.accent
        default:
            PawTheme.outline
        }
    }
}
