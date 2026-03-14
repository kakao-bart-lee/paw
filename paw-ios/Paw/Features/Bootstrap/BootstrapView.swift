import SwiftUI

struct BootstrapView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            statusOverviewCard
            AuthView()
                .environmentObject(coreManager)

            if coreManager.preview.auth.hasAccessToken {
                ChatView()
                    .environmentObject(coreManager)
                profileAndSettings
            } else {
                lockedWorkspace
            }
        }
    }

    // MARK: - Status overview

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
                NoticeCard(title: notice.title, detail: notice.detail, tone: notice.tone)
            }
        }
    }

    // MARK: - Profile & settings

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

    // MARK: - Locked workspace

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

    // MARK: - Computed text

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
}
