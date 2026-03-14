import XCTest

final class PawUITests: XCTestCase {
    private enum Identifier {
        static let title = "paw.title"
        static let currentAuthStep = "paw.auth.currentStep"
        static let shellBanner = "paw.conversations.banner"
        static let pushStatus = "paw.push.status"
        static let connectionState = "paw.runtime.connectionState"
        static let authMethodSelect = "paw.auth.button.AuthMethodSelect"
        static let phoneInput = "paw.auth.button.PhoneInput"
        static let otpVerify = "paw.auth.button.OtpVerify"
        static let deviceName = "paw.auth.button.DeviceName"
        static let usernameSetup = "paw.auth.button.UsernameSetup"
        static let authenticated = "paw.auth.button.Authenticated"
        static let sendMessage = "paw.chat.send"
        static let nextConversation = "paw.chat.nextConversation"
        static let activeLifecycle = "paw.lifecycle.active"
        static let backgroundLifecycle = "paw.lifecycle.background"
        static let registerPush = "paw.push.register"
    }

    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    func testBootstrapFlowAuthenticatesAndExercisesRuntimeSmoke() throws {
        let app = XCUIApplication()
        app.launchEnvironment["PAW_UI_TEST_MODE"] = "1"
        app.launch()

        XCTAssertTrue(app.staticTexts[Identifier.title].waitForExistence(timeout: 5))
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "1. 로그인 방식 선택")
        XCTAssertFalse(app.staticTexts[Identifier.shellBanner].exists)

        app.buttons[Identifier.phoneInput].tap()
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "2. 전화번호 입력")

        app.buttons[Identifier.otpVerify].tap()
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "3. OTP 확인")

        app.buttons[Identifier.deviceName].tap()
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "4. 디바이스 등록")
        XCTAssertEqual(app.staticTexts[Identifier.connectionState].label, "Bootstrapping")
        XCTAssertTrue(app.staticTexts[Identifier.shellBanner].label.contains("Bootstrap Crew"))

        app.buttons[Identifier.usernameSetup].tap()
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "5. username 설정")

        authCompleteButton(in: app).tap()
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "완료 · 채팅 진입 가능")
        XCTAssertTrue(app.staticTexts[Identifier.shellBanner].label.contains("Bootstrap Crew · Ready"))
        XCTAssertEqual(app.staticTexts[Identifier.connectionState].label, "Connected")

        revealIfNeeded(app.buttons[Identifier.registerPush], in: app)
        app.buttons[Identifier.registerPush].tap()
        XCTAssertEqual(app.staticTexts[Identifier.pushStatus].label, "Registered")

        revealIfNeeded(app.buttons[Identifier.nextConversation], in: app)
        app.buttons[Identifier.nextConversation].tap()
        XCTAssertTrue(app.staticTexts[Identifier.shellBanner].label.contains("Agent Ops"))

        app.buttons[Identifier.sendMessage].tap()
        XCTAssertTrue(app.staticTexts.containing(NSPredicate(format: "label CONTAINS 'Runtime live' ")).firstMatch.waitForExistence(timeout: 2))

        revealIfNeeded(app.buttons[Identifier.backgroundLifecycle], in: app)
        app.buttons[Identifier.backgroundLifecycle].tap()
        XCTAssertEqual(app.staticTexts[Identifier.connectionState].label, "Background")
        app.buttons[Identifier.activeLifecycle].tap()
        XCTAssertEqual(app.staticTexts[Identifier.connectionState].label, "Connected")

        revealAtTop(app)
        app.buttons[Identifier.authMethodSelect].tap()
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "1. 로그인 방식 선택")
        XCTAssertFalse(app.staticTexts[Identifier.shellBanner].exists)
        XCTAssertFalse(app.staticTexts[Identifier.pushStatus].exists)
    }

    private func revealIfNeeded(_ element: XCUIElement, in app: XCUIApplication) {
        guard !element.exists else { return }
        app.swipeUp()
        if !element.exists {
            app.swipeUp()
        }
    }

    private func revealAtTop(_ app: XCUIApplication) {
        for _ in 0..<3 {
            app.swipeDown()
        }
    }

    private func authCompleteButton(in app: XCUIApplication) -> XCUIElement {
        app.buttons.matching(NSPredicate(format: "identifier == %@ AND label == %@", Identifier.authenticated, "완료")).firstMatch
    }
}
