import SwiftUI

/// Legacy entry point. Delegates to Features/Bootstrap/BootstrapView.
/// Kept for backward compatibility with ContentView and Xcode project references.
struct PawBootstrapView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        BootstrapView()
            .environmentObject(coreManager)
    }
}

/// StatusPill is a shared component used across feature views.
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
