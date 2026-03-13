import SwiftUI

struct PawBootstrapView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            shellCard(
                title: "Bootstrap",
                subtitle: "keychain · APNs · runtime snapshot",
                background: PawTheme.surface2
            ) {
                metadataLine("bridge", coreManager.preview.bridgeStatus)
                metadataLine("connection", coreManager.preview.runtime.connectionState, identifier: PawAccessibility.connectionState)
                metadataLine("storage", coreManager.preview.storage.provider)
                metadataLine("push", coreManager.preview.push.status, identifier: PawAccessibility.pushStatus)
            }

            shellCard(
                title: "Auth flow",
                subtitle: "real state transitions for bootstrap shell",
                background: PawTheme.primarySoft
            ) {
                HStack(spacing: 8) {
                    authChip(title: "시작", selected: coreManager.preview.auth.step == .authMethodSelect, identifier: PawAccessibility.authButton(.authMethodSelect)) {
                        coreManager.logout()
                    }
                    authChip(title: "전화", selected: coreManager.preview.auth.step == .phoneInput, identifier: PawAccessibility.authButton(.phoneInput)) {
                        coreManager.startPhoneInput()
                    }
                    authChip(title: "OTP", selected: coreManager.preview.auth.step == .otpVerify, identifier: PawAccessibility.authButton(.otpVerify)) {
                        coreManager.submitPhone()
                    }
                    authChip(title: "기기", selected: coreManager.preview.auth.step == .deviceName, identifier: PawAccessibility.authButton(.deviceName)) {
                        coreManager.verifyOtp()
                    }
                    authChip(title: "유저", selected: coreManager.preview.auth.step == .usernameSetup, identifier: PawAccessibility.authButton(.usernameSetup)) {
                        coreManager.submitDeviceName()
                    }
                    authChip(title: "완료", selected: coreManager.preview.auth.step == .authenticated, identifier: PawAccessibility.authButton(.authenticated)) {
                        coreManager.submitUsername()
                    }
                }
                metadataLine("current step", coreManager.preview.auth.step.rawValue, identifier: PawAccessibility.currentAuthStep)
                metadataLine("phone", coreManager.preview.auth.phone.ifEmpty("(pending)"))
                metadataLine("device", coreManager.preview.auth.deviceName.ifEmpty("(pending)"))
                metadataLine("username", coreManager.preview.auth.username.ifEmpty("(pending)"))
                metadataLine("discoverable", coreManager.preview.auth.discoverableByPhone.description)
                metadataLine("has access token", coreManager.preview.auth.hasAccessToken.description)
                if let error = coreManager.preview.auth.error {
                    metadataLine("error", error)
                }
            }

            HStack(alignment: .top, spacing: 12) {
                shellCard(
                    title: "Conversations",
                    subtitle: "real bootstrap state gates the list",
                    background: PawTheme.surface1
                ) {
                    metadataLine("shell", coreManager.preview.shellBanner, identifier: PawAccessibility.shellBanner)
                    metadataLine("selected", coreManager.selectedConversation?.title ?? "(locked)")
                    ForEach(coreManager.preview.conversations) { conversation in
                        conversationRow(conversation, selected: coreManager.preview.selectedConversationID == conversation.id)
                    }
                }

                shellCard(
                    title: "Chat runtime",
                    subtitle: "runtime snapshot + streaming shell",
                    background: PawTheme.surface3
                ) {
                    metadataLine("cursors", "\(coreManager.preview.runtime.cursorCount)")
                    metadataLine("active streams", "\(coreManager.preview.runtime.activeStreamCount)")
                    metadataLine("composer", coreManager.preview.composerText, identifier: PawAccessibility.composer)
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

            HStack(spacing: 12) {
                shellCard(
                    title: "Lifecycle",
                    subtitle: "runtime hint contract",
                    background: PawTheme.sentBubble
                ) {
                    HStack(spacing: 8) {
                        authChip(title: "Active", selected: coreManager.preview.lifecycle.currentState == "Active", identifier: PawAccessibility.activeLifecycleButton) {
                            coreManager.applyLifecycle(state: "Active")
                        }
                        authChip(title: "Background", selected: coreManager.preview.lifecycle.currentState == "Background", identifier: PawAccessibility.backgroundLifecycleButton) {
                            coreManager.applyLifecycle(state: "Background")
                        }
                    }
                    metadataLine("active", coreManager.preview.lifecycle.activeHints.joined(separator: ", "))
                    metadataLine("background", coreManager.preview.lifecycle.backgroundHints.joined(separator: ", "))
                }
                shellCard(
                    title: "Platform adapters",
                    subtitle: "Keychain + APNs wiring",
                    background: PawTheme.agentBubble
                ) {
                    HStack(spacing: 8) {
                        authChip(title: "APNs 등록", selected: coreManager.preview.push.status == "Registered", identifier: PawAccessibility.registerPushButton) {
                            coreManager.registerForPush()
                        }
                        authChip(title: "APNs 해제", selected: coreManager.preview.push.status == "Unregistered", identifier: PawAccessibility.unregisterPushButton) {
                            coreManager.unregisterPush()
                        }
                    }
                    metadataLine("device key", coreManager.preview.storage.hasDeviceKey.description)
                    metadataLine("push token", coreManager.preview.push.token == nil ? "Absent" : "Present")
                    metadataLine("platform", coreManager.preview.push.platform)
                }
            }
        }
    }

    @ViewBuilder
    private func shellCard(
        title: String,
        subtitle: String,
        background: Color,
        @ViewBuilder content: () -> some View
    ) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(PawTypography.titleMedium)
                .foregroundStyle(PawTheme.strongText)
            Text(subtitle)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
            VStack(alignment: .leading, spacing: 8) {
                content()
            }
            .padding(.top, 8)
        }
        .padding(18)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(background)
        .overlay(
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .stroke(PawTheme.outline, lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 22, style: .continuous))
    }

    @ViewBuilder
    private func metadataLine(_ label: String, _ value: String, identifier: String? = nil) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(PawTypography.labelSmall)
                .foregroundStyle(PawTheme.primary)
            Text(value)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.strongText)
                .applyAccessibilityIdentifier(identifier)
        }
    }

    @ViewBuilder
    private func authChip(title: String, selected: Bool, identifier: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(title)
                .font(PawTypography.labelSmall)
                .foregroundStyle(selected ? PawTheme.background : PawTheme.strongText)
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(selected ? PawTheme.primary : PawTheme.surface3)
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
        .accessibilityIdentifier(identifier)
    }

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
                        .background(PawTheme.primary)
                        .clipShape(Capsule())
                }
            }
            .padding(12)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(selected ? PawTheme.surface4 : PawTheme.surface2)
            .overlay(
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .stroke(selected ? PawTheme.primary : PawTheme.outline, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private func messageBubble(_ message: PawMessagePreview) -> some View {
        HStack {
            if message.role == .me {
                Spacer(minLength: 24)
            }
            VStack(alignment: .leading, spacing: 4) {
                Text(message.author)
                    .font(PawTypography.labelSmall)
                    .foregroundStyle(PawTheme.primary)
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
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .stroke(PawTheme.outline, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
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
        case "primary":
            PawTheme.primary
        case "accent":
            PawTheme.accent
        default:
            PawTheme.outline
        }
    }
}

private extension View {
    @ViewBuilder
    func applyAccessibilityIdentifier(_ identifier: String?) -> some View {
        if let identifier {
            accessibilityIdentifier(identifier)
        } else {
            self
        }
    }
}

private extension String {
    func ifEmpty(_ fallback: String) -> String {
        isEmpty ? fallback : self
    }
}
