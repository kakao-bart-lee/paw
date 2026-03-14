import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Paw")
                            .font(PawTypography.headlineMedium)
                            .foregroundStyle(PawTheme.strongText)
                            .accessibilityIdentifier(PawAccessibility.title)
                        Text("Flutter 버전의 차분한 다크 메신저 분위기를 SwiftUI shell에 옮기는 기준 화면")
                            .font(PawTypography.bodyMedium)
                            .foregroundStyle(PawTheme.mutedText)
                    }

                    PawBootstrapView()

                    VStack(alignment: .leading, spacing: 8) {
                        Text("paw-core bridge")
                            .font(PawTypography.labelSmall)
                            .foregroundStyle(PawTheme.primary)
                        Text(coreManager.bindingsStatus)
                            .font(PawTypography.bodyMedium)
                            .foregroundStyle(PawTheme.strongText)
                            .accessibilityIdentifier(PawAccessibility.bridgeStatus)
                        Text("Artifacts: \(coreManager.artifactsDirectory)")
                            .font(PawTypography.bodySmall)
                            .foregroundStyle(PawTheme.mutedText)
                            .accessibilityIdentifier(PawAccessibility.artifactsDirectory)
                    }
                    .padding(18)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(PawTheme.primarySoft)
                    .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                }
                .padding(20)
            }
            .background(
                LinearGradient(
                    colors: [PawTheme.surface1, PawTheme.background],
                    startPoint: .top,
                    endPoint: .bottom
                )
                .ignoresSafeArea()
            )
            .toolbar(.hidden, for: .navigationBar)
        }
    }
}
