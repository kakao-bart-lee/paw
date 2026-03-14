import SwiftUI
#if os(iOS)
import UIKit
#endif

struct AuthView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    @State private var phoneText: String = ""
    @State private var otpText: String = ""
    @State private var deviceNameText: String = deviceNameDefault
    @State private var usernameText: String = ""
    @State private var discoverableToggle: Bool = true

    private static var deviceNameDefault: String {
        #if os(iOS)
        UIDevice.current.name
        #else
        "My Mac"
        #endif
    }

    var body: some View {
        ZStack {
            LinearGradient(
                colors: [PawTheme.background, PawTheme.backgroundGlow.opacity(0.25), PawTheme.background],
                startPoint: .top,
                endPoint: .bottom
            )
            .ignoresSafeArea()

            Circle()
                .fill(PawTheme.backgroundGlow.opacity(0.22))
                .frame(width: 260, height: 260)
                .blur(radius: 80)
                .offset(y: 40)

            VStack(spacing: 0) {
                topMetaBar
                Spacer(minLength: 0)
                content
                Spacer(minLength: 0)
                bottomActionArea
            }
            .padding(.horizontal, 30)
            .padding(.vertical, 18)
        }
    }

    private var topMetaBar: some View {
        VStack(spacing: 10) {
            HStack {
                if coreManager.preview.auth.step != .authMethodSelect {
                    Button {
                        goBack()
                    } label: {
                        Image(systemName: "arrow.left")
                            .font(.system(size: 16, weight: .regular, design: .monospaced))
                            .foregroundStyle(PawTheme.subtleText)
                    }
                    .buttonStyle(.plain)
                } else {
                    Color.clear.frame(width: 18, height: 18)
                }

                Spacer()

                Text(authProgressLabel)
                    .font(PawTypography.labelSmall)
                    .tracking(2)
                    .foregroundStyle(PawTheme.mutedText)
                    .accessibilityIdentifier(PawAccessibility.currentAuthStep)

                Spacer()

                RoundedRectangle(cornerRadius: 1)
                    .stroke(PawTheme.subtleText, lineWidth: 1)
                    .frame(width: 14, height: 8)
            }

            if coreManager.preview.auth.step != .authMethodSelect {
                HStack {
                    Spacer()
                    authChip(
                        title: "return to origin",
                        selected: false,
                        identifier: PawAccessibility.authButton(.authMethodSelect)
                    ) {
                        coreManager.logout()
                    }
                    Spacer()
                }
            }
        }
    }

    private var content: some View {
        VStack(spacing: 34) {
            switch coreManager.preview.auth.step {
            case .authMethodSelect, .phoneInput:
                authHero(title: "Paw", subtitle: "signal your presence")
                phoneEntry
            case .otpVerify:
                codeEntry
            case .deviceName:
                authHero(title: "Name the device", subtitle: "bind this presence")
                singleFieldEntry(
                    placeholder: "device name",
                    text: $deviceNameText,
                    identifier: "paw.auth.deviceNameInput"
                )
            case .usernameSetup:
                authHero(title: "Choose a signal", subtitle: "public identity")
                usernameEntry
            case .authenticated:
                authHero(title: "Presence bound", subtitle: "entering stream")
                VStack(spacing: 12) {
                    StatusPill(title: "username", value: coreManager.preview.auth.username.ifEmpty("dev"), accent: PawTheme.teal)
                    StatusPill(title: "device", value: coreManager.preview.auth.deviceName.ifEmpty(deviceNameText), accent: PawTheme.amber)
                }
            }

            if coreManager.preview.auth.isLoading {
                ProgressView()
                    .tint(PawTheme.amber)
            }

            if let error = coreManager.preview.auth.error {
                NoticeCard(title: "action needed", detail: error, tone: .warning)
                    .accessibilityIdentifier(PawAccessibility.authError)
            }
        }
        .frame(maxWidth: 280)
    }

    private func authHero(title: String, subtitle: String) -> some View {
        VStack(spacing: 10) {
            Text(title)
                .font(PawTypography.hero)
                .foregroundStyle(PawTheme.strongText)
            Text(subtitle.uppercased())
                .font(PawTypography.labelMedium)
                .tracking(4)
                .foregroundStyle(PawTheme.mutedText)
        }
    }

    private var phoneEntry: some View {
        VStack(spacing: 24) {
            singleFieldEntry(
                placeholder: "phone number",
                text: $phoneText,
                identifier: "paw.auth.phoneInput"
            )
            metadataLine("route", "sms verification")
        }
    }

    private var codeEntry: some View {
        VStack(spacing: 28) {
            Text("speak the code")
                .font(PawTypography.labelMedium)
                .tracking(4)
                .foregroundStyle(PawTheme.mutedText)

            HStack(spacing: 12) {
                ForEach(0..<6, id: \.self) { index in
                    VStack(spacing: 10) {
                        Text(character(at: index))
                            .font(PawTypography.headlineLarge)
                            .foregroundStyle(PawTheme.strongText)
                            .frame(width: 28)
                        Rectangle()
                            .fill(index == otpText.count ? PawTheme.amber : PawTheme.outline)
                            .frame(width: 34, height: 1)
                    }
                }
            }

            pawTextField(
                placeholder: "000000",
                text: $otpText,
                identifier: "paw.auth.otpInput"
            )
            #if os(iOS)
            .keyboardType(.numberPad)
            #endif

            metadataLine("phone", coreManager.preview.auth.phone.ifEmpty("+82 10-0000-0000"))
        }
    }

    private var usernameEntry: some View {
        VStack(spacing: 18) {
            singleFieldEntry(
                placeholder: "username",
                text: $usernameText,
                identifier: "paw.auth.usernameInput"
            )
            Toggle(isOn: $discoverableToggle) {
                Text("discoverable by phone")
                    .font(PawTypography.bodySmall)
                    .foregroundStyle(PawTheme.subtleText)
            }
            .tint(PawTheme.teal)
        }
    }

    private func singleFieldEntry(
        placeholder: String,
        text: Binding<String>,
        identifier: String
    ) -> some View {
        VStack(spacing: 16) {
            pawTextField(placeholder: placeholder, text: text, identifier: identifier)
        }
    }

    private var bottomActionArea: some View {
        VStack(spacing: 12) {
            switch coreManager.preview.auth.step {
            case .authMethodSelect:
                Button {
                    coreManager.startPhoneInput()
                } label: {
                    Text("transmit")
                }
                .buttonStyle(PawPrimaryButtonStyle(enabled: true))
                .accessibilityIdentifier(PawAccessibility.authButton(.phoneInput))

            case .phoneInput:
                Button {
                    if coreManager.isDevelopmentAuthBypassEnabled {
                        coreManager.devQuickLogin(phone: phoneText)
                    } else {
                        coreManager.submitPhoneAsync(phoneText)
                    }
                } label: {
                    Text("transmit")
                }
                .buttonStyle(PawPrimaryButtonStyle(enabled: isPhoneValid))
                .disabled(!isPhoneValid)
                .accessibilityIdentifier(PawAccessibility.authButton(.otpVerify))

            case .otpVerify:
                Button {
                    if coreManager.isDevelopmentAuthBypassEnabled {
                        coreManager.devQuickLogin(phone: coreManager.preview.auth.phone)
                    } else {
                        coreManager.verifyOtpAsync(otpText)
                    }
                } label: {
                    Text("return")
                }
                .buttonStyle(PawPrimaryButtonStyle(enabled: otpText.trimmingCharacters(in: .whitespacesAndNewlines).count >= 6))
                .disabled(otpText.trimmingCharacters(in: .whitespacesAndNewlines).count < 6)
                .accessibilityIdentifier(PawAccessibility.authButton(.deviceName))

            case .deviceName:
                Button {
                    if coreManager.isDevelopmentAuthBypassEnabled {
                        coreManager.devQuickLogin(phone: coreManager.preview.auth.phone, deviceName: deviceNameText)
                    } else {
                        coreManager.submitDeviceNameAsync(deviceNameText)
                    }
                } label: {
                    Text("bind")
                }
                .buttonStyle(PawPrimaryButtonStyle(enabled: !deviceNameText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty))
                .disabled(deviceNameText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                .accessibilityIdentifier(PawAccessibility.authButton(.usernameSetup))

            case .usernameSetup:
                Button {
                    coreManager.submitUsernameAsync(usernameText, discoverableByPhone: discoverableToggle)
                } label: {
                    Text("enter")
                }
                .buttonStyle(PawPrimaryButtonStyle(enabled: isUsernameValid))
                .disabled(!isUsernameValid)
                .accessibilityIdentifier(PawAccessibility.authButton(.authenticated))

                Button {
                    coreManager.skipUsernameAsync()
                } label: {
                    Text("skip")
                }
                .buttonStyle(PawSecondaryButtonStyle())
                .accessibilityIdentifier(PawAccessibility.authButton(.usernameSetup))

            case .authenticated:
                EmptyView()
            }
        }
        .frame(maxWidth: 280)
        .padding(.bottom, 18)
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

    private var isPhoneValid: Bool {
        phoneText.filter { $0.isNumber }.count >= 10
    }

    private var isUsernameValid: Bool {
        let trimmed = usernameText.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.range(of: "^[a-z0-9_]{3,20}$", options: .regularExpression) != nil
    }

    private func character(at index: Int) -> String {
        let chars = Array(otpText)
        guard index < chars.count else { return " " }
        return String(chars[index])
    }

    private func goBack() {
        switch coreManager.preview.auth.step {
        case .authMethodSelect:
            break
        case .phoneInput:
            coreManager.logout()
        case .otpVerify:
            coreManager.startPhoneInput()
        case .deviceName:
            coreManager.submitPhone(coreManager.preview.auth.phone, discoverableByPhone: coreManager.preview.auth.discoverableByPhone)
        case .usernameSetup:
            coreManager.submitPhone(coreManager.preview.auth.phone, discoverableByPhone: coreManager.preview.auth.discoverableByPhone)
            coreManager.verifyOtp("123456")
        case .authenticated:
            break
        }
    }
}

@ViewBuilder
func pawTextField(
    placeholder: String,
    text: Binding<String>,
    identifier: String
) -> some View {
    TextField(placeholder, text: text)
        .font(PawTypography.headlineMedium)
        .multilineTextAlignment(.center)
        .foregroundStyle(PawTheme.strongText)
        .padding(.vertical, 12)
        .background(Color.clear)
        .overlay(alignment: .bottom) {
            Rectangle()
                .fill(PawTheme.outline)
                .frame(height: 1)
        }
        #if os(iOS)
        .autocorrectionDisabled()
        .textInputAutocapitalization(.never)
        #endif
        .accessibilityIdentifier(identifier)
}
