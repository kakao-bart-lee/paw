import SwiftUI

struct PawBootstrapView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            shellCard(
                title: "Bootstrap",
                subtitle: "bridge · runtime snapshot · contract readiness",
                background: PawTheme.surface2
            ) {
                metadataLine("bridge", coreManager.preview.bridgeStatus)
                metadataLine("connection", coreManager.preview.runtime.connectionState)
                metadataLine("storage", coreManager.preview.storage.provider)
                metadataLine("push", coreManager.preview.push.status)
            }

            shellCard(
                title: "Auth preview",
                subtitle: "same step contract for Android / iOS",
                background: PawTheme.primarySoft
            ) {
                HStack(spacing: 8) {
                    authChip(title: "초기", selected: coreManager.preview.auth.step == "AuthMethodSelect") {
                        coreManager.resetPreview()
                    }
                    authChip(title: "전화 입력", selected: coreManager.preview.auth.step == "PhoneInput") {
                        coreManager.previewPhoneInput()
                    }
                }
                metadataLine("current step", coreManager.preview.auth.step)
                metadataLine("discoverable", coreManager.preview.auth.discoverableByPhone.description)
                metadataLine("has access token", coreManager.preview.auth.hasAccessToken.description)
            }

            HStack(spacing: 12) {
                shellCard(
                    title: "Lifecycle",
                    subtitle: "active/background runtime hints",
                    background: PawTheme.sentBubble
                ) {
                    metadataLine("active", coreManager.preview.lifecycle.activeHints.joined(separator: ", "))
                    metadataLine("background", coreManager.preview.lifecycle.backgroundHints.joined(separator: ", "))
                }
                shellCard(
                    title: "Next app work",
                    subtitle: "platform adapters + auth/bootstrap",
                    background: PawTheme.agentBubble
                ) {
                    metadataLine("1", "Keychain token vault")
                    metadataLine("2", "APNs registrar")
                    metadataLine("3", "real bootstrap wiring")
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
