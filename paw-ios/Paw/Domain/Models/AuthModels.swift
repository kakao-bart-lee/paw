import Foundation

enum PawAuthStep: String, CaseIterable {
    case authMethodSelect = "AuthMethodSelect"
    case phoneInput = "PhoneInput"
    case otpVerify = "OtpVerify"
    case deviceName = "DeviceName"
    case usernameSetup = "UsernameSetup"
    case authenticated = "Authenticated"
}

struct PawAuthPreview {
    var step: PawAuthStep
    var phone: String
    var deviceName: String
    var username: String
    var discoverableByPhone: Bool
    var hasSessionToken: Bool
    var hasAccessToken: Bool
    var hasRefreshToken: Bool
    var isLoading: Bool
    var error: String?
}

struct PawStoredTokens: Equatable {
    var sessionToken: String?
    var accessToken: String?
    var refreshToken: String?
}
