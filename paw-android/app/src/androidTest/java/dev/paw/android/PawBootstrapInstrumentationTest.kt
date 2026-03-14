package dev.paw.android

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import org.junit.Rule
import org.junit.Test

class PawBootstrapInstrumentationTest {
    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun launch_showsBootstrapShell() {
        composeTestRule.onNodeWithTag(PawTestTags.APP_TITLE).assertIsDisplayed()
        composeTestRule.onNodeWithTag(PawTestTags.AUTH_STEP_VALUE).assertIsDisplayed()
        composeTestRule.onNodeWithTag(PawTestTags.AUTH_CONTINUE_PHONE).assertIsDisplayed()
    }

    @Test
    fun continueWithPhone_transitionsToPhoneInput() {
        composeTestRule.onNodeWithTag(PawTestTags.AUTH_CONTINUE_PHONE).performClick()

        composeTestRule.onNodeWithTag(PawTestTags.AUTH_PHONE_INPUT).assertIsDisplayed()
        composeTestRule.onNodeWithTag(PawTestTags.AUTH_REQUEST_OTP).assertIsDisplayed()
    }

    @Test
    fun phoneChip_returnsToPhoneInputAfterReset() {
        composeTestRule.onNodeWithTag(PawTestTags.AUTH_CONTINUE_PHONE).performClick()
        composeTestRule.onNodeWithTag(PawTestTags.AUTH_CHIP_RESET).performClick()
        composeTestRule.onNodeWithTag(PawTestTags.AUTH_CHIP_PHONE).performClick()

        composeTestRule.onNodeWithTag(PawTestTags.AUTH_PHONE_INPUT).assertIsDisplayed()
        composeTestRule.onNodeWithTag(PawTestTags.AUTH_STEP_VALUE).assertIsDisplayed()
    }
}
