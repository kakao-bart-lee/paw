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
                metadataLine("connection", coreManager.preview.runtime.connectionState)
                metadataLine("storage", coreManager.preview.storage.provider)
                metadataLine("push", coreManager.preview.push.status)
            }

            shellCard(
                title: "Auth flow",
                subtitle: "real state transitions for bootstrap shell",
                background: PawTheme.primarySoft
            ) {
                HStack(spacing: 8) {
                    authChip(title: "시작", selected: coreManager.preview.auth.step == .authMethodSelect) {
                        coreManager.logout()
                    }
                    authChip(title: "전화", selected: coreManager.preview.auth.step == .phoneInput) {
                        coreManager.startPhoneInput()
                    }
                    authChip(title: "OTP", selected: coreManager.preview.auth.step == .otpVerify) {
                        coreManager.submitPhone()
                    }
                    authChip(title: "기기", selected: coreManager.preview.auth.step == .deviceName) {
                        coreManager.verifyOtp()
                    }
                    authChip(title: "유저", selected: coreManager.preview.auth.step == .usernameSetup) {
                        coreManager.submitDeviceName()
                    }
                    authChip(title: "완료", selected: coreManager.preview.auth.step == .authenticated) {
                        coreManager.submitUsername()
                    }
                }
                metadataLine("current step", coreManager.preview.auth.step.rawValue)
                metadataLine("phone", coreManager.preview.auth.phone.ifEmpty("(pending)"))
                metadataLine("device", coreManager.preview.auth.deviceName.ifEmpty("(pending)"))
                metadataLine("username", coreManager.preview.auth.username.ifEmpty("(pending)"))
                metadataLine("discoverable", coreManager.preview.auth.discoverableByPhone.description)
                metadataLine("has access token", coreManager.preview.auth.hasAccessToken.description)
                if let error = coreManager.preview.auth.error {
                    metadataLine("error", error)
                }
            }

            HStack(spacing: 12) {
                shellCard(
                    title: "Lifecycle",
                    subtitle: "runtime hint contract",
                    background: PawTheme.sentBubble
                ) {
                    HStack(spacing: 8) {
                        authChip(title: "Active", selected: coreManager.preview.lifecycle.currentState == "Active") {
                            coreManager.applyLifecycle(state: "Active")
                        }
                        authChip(title: "Background", selected: coreManager.preview.lifecycle.currentState == "Background") {
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
                        authChip(title: "APNs 등록", selected: coreManager.preview.push.status == "Registered") {
                            coreManager.registerForPush()
                        }
                        authChip(title: "APNs 해제", selected: coreManager.preview.push.status == "Unregistered") {
                            coreManager.unregisterPush()
                        }
                    }
                    metadataLine("device key", coreManager.preview.storage.hasDeviceKey.description)
                    metadataLine("push token", coreManager.preview.push.token ?? "(unregistered)")
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
    private func metadataLine(_ label: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(PawTypography.labelSmall)
                .foregroundStyle(PawTheme.primary)
            Text(value)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.strongText)
        }
    }

    @ViewBuilder
    private func authChip(title: String, selected: Bool, action: @escaping () -> Void) -> some View {
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
    }
}

private extension String {
    func ifEmpty(_ fallback: String) -> String {
        isEmpty ? fallback : self
    }
}
