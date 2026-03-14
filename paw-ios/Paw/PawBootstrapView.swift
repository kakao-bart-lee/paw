import SwiftUI

struct PawBootstrapView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        let showDiagnostics = coreManager.preview.auth.step == .authenticated
            || coreManager.preview.auth.step == .usernameSetup
            || coreManager.preview.auth.step == .deviceName

        VStack(alignment: .leading, spacing: 14) {
            if showDiagnostics {
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
            } else {
                shellCard(
                    title: "Ready for sign-in",
                    subtitle: "local iOS shell + shared auth contract",
                    background: PawTheme.surface2
                ) {
                    metadataLine("storage", coreManager.preview.storage.provider)
                    metadataLine("device key", coreManager.preview.storage.hasDeviceKey.description)
                    metadataLine("dev otp", "137900")
                }
            }

            shellCard(
                title: authCardTitle,
                subtitle: authCardSubtitle,
                background: PawTheme.primarySoft
            ) {
                HStack(spacing: 8) {
                    if coreManager.preview.auth.step != .authMethodSelect {
                        authChip(title: "처음부터", selected: false, identifier: PawAccessibility.authButton(.authMethodSelect)) {
                            coreManager.logout()
                        }
                    }
                    if coreManager.preview.auth.step != .phoneInput && coreManager.preview.auth.step != .authMethodSelect {
                        authChip(title: "전화 입력", selected: false, identifier: PawAccessibility.authButton(.phoneInput)) {
                            coreManager.startPhoneInput()
                        }
                    }
                    if coreManager.preview.auth.step != .authMethodSelect {
                        authChip(title: "새로고침", selected: false, identifier: PawAccessibility.authButton(.authenticated)) {
                            coreManager.refresh()
                        }
                    }
                }
                Text(authProgressLabel)
                    .font(PawTypography.titleMedium)
                    .foregroundStyle(PawTheme.strongText)
                    .accessibilityIdentifier(PawAccessibility.currentAuthStep)
                authStepBody
                if let error = coreManager.preview.auth.error {
                    metadataLine("error", error)
                }
            }

            if showDiagnostics {
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
    }

    private var authCardTitle: String {
        switch coreManager.preview.auth.step {
        case .authMethodSelect: "Sign in"
        case .phoneInput: "Phone verification"
        case .otpVerify: "OTP verification"
        case .deviceName: "Device registration"
        case .usernameSetup: "Finish profile"
        case .authenticated: "Authenticated"
        }
    }

    private var authCardSubtitle: String {
        switch coreManager.preview.auth.step {
        case .authMethodSelect: "start with the same OTP flow used in Flutter"
        case .phoneInput: "enter the number that will receive the OTP"
        case .otpVerify: "confirm the code, then unlock native bootstrap"
        case .deviceName: "register this device and enable session restore"
        case .usernameSetup: "choose a username for profile/search"
        case .authenticated: "chat runtime and push wiring are now available"
        }
    }

    private var authProgressLabel: String {
        switch coreManager.preview.auth.step {
        case .authMethodSelect: "1. 로그인 방식 선택"
        case .phoneInput: "2. 전화번호 입력"
        case .otpVerify: "3. OTP 확인"
        case .deviceName: "4. 디바이스 등록"
        case .usernameSetup: "5. username 설정"
        case .authenticated: "완료 · 채팅 진입 가능"
        }
    }

    @ViewBuilder
    private var authStepBody: some View {
        switch coreManager.preview.auth.step {
        case .authMethodSelect:
            VStack(alignment: .leading, spacing: 10) {
                Text("전화번호로 시작하기")
                    .font(PawTypography.headlineMedium)
                    .foregroundStyle(PawTheme.strongText)
                Text("기존 사용자도 같은 OTP 흐름으로 바로 로그인할 수 있습니다. iOS에서는 이 흐름을 먼저 안정화합니다.")
                    .font(PawTypography.bodyMedium)
                    .foregroundStyle(PawTheme.mutedText)
                authChip(title: "전화번호로 계속", selected: false, identifier: PawAccessibility.authButton(.phoneInput)) {
                    coreManager.startPhoneInput()
                }
            }
        case .phoneInput:
            metadataLine("phone", coreManager.preview.auth.phone.ifEmpty("(pending)"))
            Text("한국 번호는 +82 기준으로 처리합니다.")
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
            authChip(title: "OTP로 진행", selected: false, identifier: PawAccessibility.authButton(.otpVerify)) {
                coreManager.submitPhone()
            }
        case .otpVerify:
            metadataLine("phone", coreManager.preview.auth.phone.ifEmpty("(pending)"))
            metadataLine("dev otp", "137900")
            authChip(title: "OTP 확인", selected: false, identifier: PawAccessibility.authButton(.deviceName)) {
                coreManager.verifyOtp("137900")
            }
        case .deviceName:
            metadataLine("device", coreManager.preview.auth.deviceName.ifEmpty("(pending)"))
            authChip(title: "디바이스 등록", selected: false, identifier: PawAccessibility.authButton(.usernameSetup)) {
                coreManager.submitDeviceName()
            }
        case .usernameSetup:
            metadataLine("username", coreManager.preview.auth.username.ifEmpty("(pending)"))
            metadataLine("discoverable", coreManager.preview.auth.discoverableByPhone.description)
            HStack(spacing: 8) {
                authChip(title: "완료", selected: false, identifier: PawAccessibility.authButton(.authenticated)) {
                    coreManager.submitUsername()
                }
                authChip(title: "건너뛰기", selected: false, identifier: PawAccessibility.authButton(.usernameSetup)) {
                    coreManager.skipUsername()
                }
            }
        case .authenticated:
            metadataLine("username", coreManager.preview.auth.username.ifEmpty("(unset)"))
            metadataLine("device", coreManager.preview.auth.deviceName.ifEmpty("(pending)"))
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
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(PawTheme.outline, lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
    }

    @ViewBuilder
    private func metadataLine(_ label: String, _ value: String, identifier: String? = nil) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(PawTypography.labelSmall)
                .foregroundStyle(PawTheme.accent)
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
                .background(selected ? PawTheme.accent : PawTheme.surface3)
                .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))
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
        case "primary":
            PawTheme.accent
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
