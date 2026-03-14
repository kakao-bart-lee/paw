import SwiftUI

struct AuthView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
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
                NoticeCard(title: "Action needed", detail: error, tone: .warning)
                    .accessibilityIdentifier(PawAccessibility.authError)
            }
        }
    }

    // MARK: - Computed labels

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

    // MARK: - Step body

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
}

// MARK: - Shared view helpers (used by AuthView, ChatView, BootstrapView)

@ViewBuilder
func shellCard(
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
func metadataLine(_ label: String, _ value: String, identifier: String? = nil) -> some View {
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
func metadataPillRow(_ pills: StatusPill...) -> some View {
    HStack(spacing: 8) {
        ForEach(Array(pills.enumerated()), id: \.offset) { _, pill in
            pill
        }
    }
}

@ViewBuilder
func emptyState(title: String, detail: String, identifier: String) -> some View {
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

enum NoticeTone {
    case info
    case warning
}

struct NoticeCard: View {
    let title: String
    let detail: String
    let tone: NoticeTone

    var body: some View {
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
}

@ViewBuilder
func authChip(title: String, selected: Bool, identifier: String, action: @escaping () -> Void) -> some View {
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
