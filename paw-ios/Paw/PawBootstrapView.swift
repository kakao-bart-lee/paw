import SwiftUI

struct StatusPill: View {
    let title: String
    let value: String
    var identifier: String? = nil

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title.uppercased())
                .font(PawTypography.labelSmall)
                .foregroundStyle(PawTheme.mutedText)
            Text(value)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.strongText)
                .lineLimit(2)
                .applyAccessibilityIdentifier(identifier)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(PawTheme.surface3)
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(PawTheme.outline, lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
    }
}

struct PawBootstrapView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            statusOverviewCard
            authCard

            if coreManager.preview.auth.hasAccessToken {
                conversationWorkspace
                profileAndSettings
            } else {
                lockedWorkspace
            }
        }
    }

    private var statusOverviewCard: some View {
        shellCard(
            title: coreManager.preview.auth.hasAccessToken ? "Workspace" : "Native readiness",
            subtitle: coreManager.preview.auth.hasAccessToken
                ? "post-auth inbox, chat, profile, and runtime controls"
                : "sign in, restore session, then unlock the iOS chat workspace",
            background: PawTheme.surface2
        ) {
            metadataPillRow(
                StatusPill(title: "bridge", value: coreManager.preview.bridgeStatus),
                StatusPill(title: "runtime", value: coreManager.preview.runtime.connectionState, identifier: PawAccessibility.connectionState),
                StatusPill(title: "push", value: coreManager.preview.push.status, identifier: coreManager.preview.auth.hasAccessToken ? PawAccessibility.pushStatus : nil)
            )

            if coreManager.preview.auth.hasAccessToken {
                VStack(alignment: .leading, spacing: 8) {
                    Text(coreManager.preview.shellBanner)
                        .font(PawTypography.titleMedium)
                        .foregroundStyle(PawTheme.strongText)
                        .accessibilityIdentifier(PawAccessibility.shellBanner)
                    Text(workspaceStatusCopy)
                        .font(PawTypography.bodySmall)
                        .foregroundStyle(PawTheme.mutedText)
                }
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Sign in to open conversations, restore the last runtime snapshot, and continue from the same warm editorial shell used elsewhere.")
                        .font(PawTypography.bodyMedium)
                        .foregroundStyle(PawTheme.strongText)
                    Text("Keychain and APNs stay wired in the background; the UI stays focused on the next action instead of raw diagnostics.")
                        .font(PawTypography.bodySmall)
                        .foregroundStyle(PawTheme.mutedText)
                }
            }

            if let notice = inlineNotice {
                noticeCard(title: notice.title, detail: notice.detail, tone: notice.tone)
            }
        }
    }

    private var authCard: some View {
        shellCard(
            title: authCardTitle,
            subtitle: authCardSubtitle,
            background: PawTheme.primarySoft
        ) {
            Text(authProgressLabel)
                .font(PawTypography.headlineMedium)
                .foregroundStyle(PawTheme.strongText)
                .accessibilityIdentifier(PawAccessibility.currentAuthStep)

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

            authStepBody

            if let error = coreManager.preview.auth.error {
                noticeCard(title: "Action needed", detail: error, tone: .warning)
                    .accessibilityIdentifier(PawAccessibility.authError)
            }
        }
    }

    private var conversationWorkspace: some View {
        VStack(alignment: .leading, spacing: 12) {
            conversationListSection
            chatWorkspace
        }
    }

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

    private var profileAndSettings: some View {
        HStack(alignment: .top, spacing: 12) {
            shellCard(
                title: "Profile",
                subtitle: "identity and handoff state for search + chat",
                background: PawTheme.surface3
            ) {
                metadataLine("username", coreManager.preview.auth.username.ifEmpty("ready to claim"))
                metadataLine("phone", coreManager.preview.auth.phone.ifEmpty("stored after verification"))
                metadataLine("discoverable", coreManager.preview.auth.discoverableByPhone ? "Enabled" : "Private")
                Text("Your chat identity is restored into the active composer and thread history.")
                    .font(PawTypography.bodySmall)
                    .foregroundStyle(PawTheme.mutedText)
                    .accessibilityIdentifier(PawAccessibility.profileSummary)
            }

            shellCard(
                title: "Settings",
                subtitle: "device, push, and lifecycle health",
                background: PawTheme.agentBubble
            ) {
                metadataLine("device", coreManager.preview.auth.deviceName.ifEmpty("pending"))
                metadataLine("push", coreManager.preview.push.status, identifier: PawAccessibility.pushStatus)
                metadataLine("storage", coreManager.preview.storage.hasDeviceKey ? "Keychain ready" : "Device key pending")
                Text(settingsSummaryText)
                    .font(PawTypography.bodySmall)
                    .foregroundStyle(PawTheme.mutedText)
                    .accessibilityIdentifier(PawAccessibility.settingsSummary)
                HStack(spacing: 8) {
                    authChip(title: "APNs 등록", selected: coreManager.preview.push.status == "Registered", identifier: PawAccessibility.registerPushButton) {
                        coreManager.registerForPush()
                    }
                    authChip(title: "APNs 해제", selected: coreManager.preview.push.status == "Unregistered", identifier: PawAccessibility.unregisterPushButton) {
                        coreManager.unregisterPush()
                    }
                }
                HStack(spacing: 8) {
                    authChip(title: "Active", selected: coreManager.preview.lifecycle.currentState == "Active", identifier: PawAccessibility.activeLifecycleButton) {
                        coreManager.applyLifecycle(state: "Active")
                    }
                    authChip(title: "Background", selected: coreManager.preview.lifecycle.currentState == "Background", identifier: PawAccessibility.backgroundLifecycleButton) {
                        coreManager.applyLifecycle(state: "Background")
                    }
                }
            }
        }
    }

    private var lockedWorkspace: some View {
        shellCard(
            title: "Workspace locked",
            subtitle: "conversations and composer unlock after verification",
            background: PawTheme.surface1
        ) {
            emptyState(
                title: "Unlock chat workspace",
                detail: "Complete OTP, register this device, then finish the profile step to restore conversations and enable the composer.",
                identifier: PawAccessibility.chatEmpty
            )
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
                Text("기존 사용자도 같은 OTP 흐름으로 바로 로그인할 수 있습니다. iOS에서는 foundation shell 대신 실제 대화 진입 흐름을 우선 다듬습니다.")
                    .font(PawTypography.bodyMedium)
                    .foregroundStyle(PawTheme.mutedText)
                authChip(title: "전화번호로 계속", selected: false, identifier: PawAccessibility.authButton(.phoneInput)) {
                    coreManager.startPhoneInput()
                }
            }
        case .phoneInput:
            metadataLine("phone", coreManager.preview.auth.phone.ifEmpty("+82 10-5555-0101"))
            Text("한국 번호는 +82 기준으로 처리하며, 확인 후 바로 OTP 단계로 이어집니다.")
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
            authChip(title: "OTP로 진행", selected: false, identifier: PawAccessibility.authButton(.otpVerify)) {
                coreManager.submitPhone()
            }
        case .otpVerify:
            metadataLine("phone", coreManager.preview.auth.phone.ifEmpty("+82 10-5555-0101"))
            metadataLine("dev otp", "137900")
            Text("OTP 확인이 끝나면 저장된 세션과 대화 shell을 바로 복구할 준비를 시작합니다.")
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
            authChip(title: "OTP 확인", selected: false, identifier: PawAccessibility.authButton(.deviceName)) {
                coreManager.verifyOtp("137900")
            }
        case .deviceName:
            metadataLine("device", coreManager.preview.auth.deviceName.ifEmpty("Haruna's iPhone"))
            Text("이 기기 이름은 세션 복구와 push 연결 상태를 설명하는 설정 요약에도 사용됩니다.")
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
            authChip(title: "디바이스 등록", selected: false, identifier: PawAccessibility.authButton(.usernameSetup)) {
                coreManager.submitDeviceName()
            }
        case .usernameSetup:
            metadataLine("username", coreManager.preview.auth.username.ifEmpty("haruna"))
            metadataLine("discoverable", coreManager.preview.auth.discoverableByPhone ? "Enabled" : "Private")
            Text("프로필을 마치면 바로 conversations, composer, settings 요약이 열립니다.")
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
            HStack(spacing: 8) {
                authChip(title: "완료", selected: false, identifier: PawAccessibility.authButton(.authenticated)) {
                    coreManager.submitUsername()
                }
                authChip(title: "건너뛰기", selected: false, identifier: PawAccessibility.authButton(.usernameSetup)) {
                    coreManager.skipUsername()
                }
            }
        case .authenticated:
            VStack(alignment: .leading, spacing: 8) {
                Text("Everything is ready for chat.")
                    .font(PawTypography.titleMedium)
                    .foregroundStyle(PawTheme.strongText)
                Text("Switch threads, send a runtime prompt, and verify push or lifecycle behavior without dropping the current shell context.")
                    .font(PawTypography.bodySmall)
                    .foregroundStyle(PawTheme.mutedText)
                metadataPillRow(
                    StatusPill(title: "username", value: coreManager.preview.auth.username.ifEmpty("guest")),
                    StatusPill(title: "device", value: coreManager.preview.auth.deviceName.ifEmpty("ready")),
                    StatusPill(title: "composer", value: coreManager.preview.composerText, identifier: PawAccessibility.composer)
                )
            }
        }
    }

    private var workspaceStatusCopy: String {
        switch coreManager.preview.runtime.connectionState {
        case "Bootstrapping":
            "Session restored. We are wiring the device and preparing the live inbox before full chat entry."
        case "Background":
            "Realtime is paused while the app is backgrounded. Resume to continue streaming replies."
        case "Connected", "Ready":
            "Conversation history, composer prompts, and runtime actions are ready for product-level QA."
        default:
            "Runtime will reconnect after authentication."
        }
    }

    private var chatSummaryCopy: String {
        "\(coreManager.preview.messages.count) messages · \(coreManager.preview.runtime.activeStreamCount) active streams · selected thread \(coreManager.selectedConversation?.title ?? "none")"
    }

    private var settingsSummaryText: String {
        "\(coreManager.preview.lifecycle.currentState) · active hints: \(coreManager.preview.lifecycle.activeHints.joined(separator: ", ")) · background hints: \(coreManager.preview.lifecycle.backgroundHints.joined(separator: ", "))"
    }

    private var inlineNotice: (title: String, detail: String, tone: NoticeTone)? {
        if let error = coreManager.preview.auth.error {
            return ("Action needed", error, .warning)
        }
        if coreManager.preview.runtime.connectionState == "Bootstrapping" {
            return ("Preparing workspace", "Session restore succeeded. Finish device/profile setup to open the full chat workspace.", .info)
        }
        if coreManager.preview.runtime.connectionState == "Background" {
            return ("Backgrounded", "Streaming is paused until the app returns to the foreground.", .info)
        }
        return nil
    }

    private enum NoticeTone {
        case info
        case warning
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
    private func metadataPillRow(_ pills: StatusPill...) -> some View {
        HStack(spacing: 8) {
            ForEach(Array(pills.enumerated()), id: \.offset) { _, pill in
                pill
            }
        }
    }

    @ViewBuilder
    private func emptyState(title: String, detail: String, identifier: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(PawTypography.titleMedium)
                .foregroundStyle(PawTheme.strongText)
            Text(detail)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(PawTheme.surface2)
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(PawTheme.outline, style: StrokeStyle(lineWidth: 1, dash: [4, 4]))
        )
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
        .accessibilityIdentifier(identifier)
    }

    @ViewBuilder
    private func noticeCard(title: String, detail: String, tone: NoticeTone) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(PawTypography.titleMedium)
                .foregroundStyle(PawTheme.strongText)
            Text(detail)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(tone == .warning ? PawTheme.sentBubble : PawTheme.surface3)
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(tone == .warning ? PawTheme.accent : PawTheme.outline, lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
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
        case "primary", "accent":
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
