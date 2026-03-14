import SwiftUI

struct PawBootstrapView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        BootstrapView()
            .environmentObject(coreManager)
    }
}

struct StatusPill: View {
    let title: String
    let value: String
    var accent: Color = PawTheme.outline
    var identifier: String? = nil

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title.uppercased())
                .font(PawTypography.labelSmall)
                .tracking(2)
                .foregroundStyle(PawTheme.mutedText)
            Text(value)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.strongText)
                .applyAccessibilityIdentifier(identifier)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .overlay(
            Rectangle()
                .stroke(accent.opacity(0.18), lineWidth: 1)
        )
    }
}

extension View {
    @ViewBuilder
    func applyAccessibilityIdentifier(_ identifier: String?) -> some View {
        if let identifier {
            accessibilityIdentifier(identifier)
        } else {
            self
        }
    }
}

extension String {
    func ifEmpty(_ fallback: String) -> String {
        isEmpty ? fallback : self
    }
}

@ViewBuilder
func shellCard(
    title: String? = nil,
    subtitle: String? = nil,
    background: Color = .clear,
    padding: CGFloat = 0,
    @ViewBuilder content: () -> some View
) -> some View {
    VStack(alignment: .leading, spacing: 8) {
        if let title {
            Text(title)
                .font(PawTypography.labelMedium)
                .tracking(3)
                .textCase(.uppercase)
                .foregroundStyle(PawTheme.mutedText)
        }
        if let subtitle {
            Text(subtitle)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
        }
        content()
    }
    .padding(padding)
    .background(background)
}

@ViewBuilder
func metadataLine(_ label: String, _ value: String, identifier: String? = nil) -> some View {
    HStack(spacing: 8) {
        Text(label.uppercased())
            .font(PawTypography.labelSmall)
            .tracking(2)
            .foregroundStyle(PawTheme.mutedText)
        Text(value)
            .font(PawTypography.bodySmall)
            .foregroundStyle(PawTheme.strongText)
            .applyAccessibilityIdentifier(identifier)
    }
}

@ViewBuilder
func metadataPillRow(_ pills: StatusPill...) -> some View {
    HStack(spacing: 10) {
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
    .padding(.vertical, 12)
    .accessibilityIdentifier(identifier)
}

enum NoticeTone {
    case info
    case warning
    case success
}

struct NoticeCard: View {
    let title: String
    let detail: String
    let tone: NoticeTone

    private var tint: Color {
        switch tone {
        case .info: PawTheme.teal
        case .warning: PawTheme.amber
        case .success: PawTheme.success
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title.uppercased())
                .font(PawTypography.labelSmall)
                .tracking(2)
                .foregroundStyle(tint)
            Text(detail)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.subtleText)
        }
        .padding(.vertical, 10)
        .padding(.horizontal, 12)
        .background(tint.opacity(0.06))
        .overlay(
            Rectangle()
                .stroke(tint.opacity(0.18), lineWidth: 1)
        )
    }
}

@ViewBuilder
func authChip(title: String, selected: Bool, identifier: String, action: @escaping () -> Void) -> some View {
    Button(action: action) {
        Text(title)
            .font(PawTypography.labelSmall)
            .tracking(2)
            .textCase(.uppercase)
            .foregroundStyle(selected ? PawTheme.strongText : PawTheme.mutedText)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .overlay(
                Rectangle()
                    .stroke(selected ? PawTheme.teal.opacity(0.25) : PawTheme.outline, lineWidth: 1)
            )
    }
    .buttonStyle(.plain)
    .accessibilityIdentifier(identifier)
}

struct PawPrimaryButtonStyle: ButtonStyle {
    let enabled: Bool

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(PawTypography.labelMedium)
            .tracking(3)
            .textCase(.uppercase)
            .foregroundStyle(enabled ? PawTheme.strongText : PawTheme.mutedText.opacity(0.45))
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .overlay(
                Rectangle()
                    .stroke(enabled ? PawTheme.amber.opacity(0.45) : PawTheme.outline, lineWidth: 1)
            )
            .background(PawTheme.background)
            .opacity(configuration.isPressed && enabled ? 0.72 : 1)
    }
}

struct PawSecondaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(PawTypography.labelMedium)
            .tracking(3)
            .textCase(.uppercase)
            .foregroundStyle(PawTheme.mutedText)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .overlay(
                Rectangle()
                    .stroke(PawTheme.outline, lineWidth: 1)
            )
            .opacity(configuration.isPressed ? 0.72 : 1)
    }
}
