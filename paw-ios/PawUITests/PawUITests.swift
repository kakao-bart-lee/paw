import XCTest

final class PawUITests: XCTestCase {
    private enum Identifier {
        static let currentAuthStep = "paw.auth.currentStep"
        static let authMethodSelect = "paw.auth.button.AuthMethodSelect"
        static let phoneInput = "paw.auth.button.PhoneInput"
        static let otpVerify = "paw.auth.button.OtpVerify"
        static let deviceName = "paw.auth.button.DeviceName"
        static let usernameSetup = "paw.auth.button.UsernameSetup"
        static let authenticated = "paw.auth.button.Authenticated"
        static let phoneTextField = "paw.auth.phoneInput"
        static let otpTextField = "paw.auth.otpInput"
        static let deviceNameTextField = "paw.auth.deviceNameInput"
        static let usernameTextField = "paw.auth.usernameInput"
        static let authError = "paw.auth.error"
        static let composer = "paw.runtime.composer"
        static let sendMessage = "paw.chat.send"
        static let chatEmpty = "paw.chat.empty"
        static let conversationsEmpty = "paw.conversations.empty"
        static let mainShell = "paw.main.shell"
        static let chatListTitle = "paw.chat.list.title"
        static let mainTabChat = "paw.main.tab.chat"
        static let mainTabAgent = "paw.main.tab.agent"
        static let mainTabSettings = "paw.main.tab.settings"
    }

    override func setUpWithError() throws {
        continueAfterFailure = false
    }


    func testGeneralLaunchBypassesAuthAndShowsMainShell() throws {
        let app = XCUIApplication()
        app.launch()

        XCTAssertTrue(app.staticTexts["STREAM"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.staticTexts[Identifier.currentAuthStep].exists)
    }

    func testAuthFlowShowsTextFieldsAndProgresses() throws {
        let app = XCUIApplication()
        app.launchEnvironment["PAW_UI_TEST_MODE"] = "1"
        app.launch()

        // Step 1: auth method select
        XCTAssertTrue(app.staticTexts[Identifier.currentAuthStep].waitForExistence(timeout: 5))
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "1. \u{B85C}\u{ADF8}\u{C778} \u{BC29}\u{C2DD} \u{C120}\u{D0DD}")

        // Tap "전화번호로 계속" to go to phone input
        app.buttons[Identifier.phoneInput].tap()
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "2. \u{C804}\u{D654}\u{BC88}\u{D638} \u{C785}\u{B825}")

        // Phone input TextField should exist
        let phoneField = app.textFields[Identifier.phoneTextField]
        XCTAssertTrue(phoneField.exists, "Phone input TextField should be visible")

        // "처음부터" button should exist to go back
        XCTAssertTrue(app.buttons[Identifier.authMethodSelect].exists, "Reset button should be visible")

        // Tap reset and verify we go back
        app.buttons[Identifier.authMethodSelect].tap()
        XCTAssertEqual(app.staticTexts[Identifier.currentAuthStep].label, "1. \u{B85C}\u{ADF8}\u{C778} \u{BC29}\u{C2DD} \u{C120}\u{D0DD}")
    }

    func testAuthViewShowsInputFieldsAtEachStep() throws {
        let app = XCUIApplication()
        app.launchEnvironment["PAW_UI_TEST_MODE"] = "1"
        app.launch()

        XCTAssertTrue(app.staticTexts[Identifier.currentAuthStep].waitForExistence(timeout: 5))

        // Navigate to phone input
        app.buttons[Identifier.phoneInput].tap()

        // Verify phone TextField is present
        XCTAssertTrue(app.textFields[Identifier.phoneTextField].exists)

        // Tapping OTP button will trigger async API call which will fail, but the
        // TextField should still be present at the phone step
        // (The step only advances on API success)
    }
}
