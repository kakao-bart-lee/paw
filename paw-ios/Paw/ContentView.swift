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
                        Text("Flutter 버전의 차분한 다크 메신저 분위기를 SwiftUI shell에 옮기는 기준 화면")
                            .font(PawTypography.bodyMedium)
                            .foregroundStyle(PawTheme.mutedText)
                    }

                    shellCard(title: "대화 목록", subtitle: "rounded cards · thin outline", background: PawTheme.surface2)

                    HStack(spacing: 12) {
                        shellCard(title: "보낸 메시지", subtitle: "primary 강조", background: PawTheme.sentBubble)
                        shellCard(title: "AI 스트림", subtitle: "agent bubble", background: PawTheme.agentBubble)
                    }

                    VStack(alignment: .leading, spacing: 8) {
                        Text("paw-core bridge")
                            .font(PawTypography.labelSmall)
                            .foregroundStyle(PawTheme.primary)
                        Text(coreManager.bindingsStatus)
                            .font(PawTypography.bodyMedium)
                            .foregroundStyle(PawTheme.strongText)
                        Text("Artifacts: \(coreManager.artifactsDirectory)")
                            .font(PawTypography.bodySmall)
                            .foregroundStyle(PawTheme.mutedText)
                    }
                    .padding(18)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(PawTheme.primarySoft)
                    .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
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
            .navigationTitle("Paw")
        }
    }

    @ViewBuilder
    private func shellCard(title: String, subtitle: String, background: Color) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(PawTypography.titleMedium)
                .foregroundStyle(PawTheme.strongText)
            Text(subtitle)
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
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
}
