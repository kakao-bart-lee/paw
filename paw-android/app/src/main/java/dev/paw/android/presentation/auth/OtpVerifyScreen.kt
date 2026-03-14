package dev.paw.android.presentation.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawStrongText
import uniffi.paw_core.AuthStepView

@Composable
fun OtpVerifyScreen(navController: NavController, viewModel: BootstrapViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val authVm = viewModel.authViewModel
    val otpValue = uiState.otpInput

    // Navigate on step change
    LaunchedEffect(uiState.preview.auth.step) {
        when (uiState.preview.auth.step) {
            AuthStepView.DEVICE_NAME -> navController.navigate(PawRoutes.DEVICE_REGISTER)
            AuthStepView.USERNAME_SETUP -> navController.navigate(PawRoutes.USERNAME_SETUP)
            AuthStepView.AUTHENTICATED -> navController.navigate(PawRoutes.CHAT_LIST) {
                popUpTo(0) { inclusive = true }
            }
            else -> {}
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(PawBackground)
            .statusBarsPadding(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Spacer(Modifier.weight(1f))

        Text(
            "SPEAK THE CODE",
            style = MaterialTheme.typography.labelLarge,
            color = PawMutedText,
        )

        Spacer(Modifier.height(48.dp))

        // OTP digits display
        Box(contentAlignment = Alignment.Center) {
            BasicTextField(
                value = otpValue,
                onValueChange = { newValue ->
                    if (newValue.length <= 6 && newValue.all { it.isDigit() }) {
                        authVm.onOtpChanged(newValue)
                        if (newValue.length == 6) {
                            authVm.verifyOtp()
                        }
                    }
                },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
                textStyle = MaterialTheme.typography.headlineLarge.copy(color = PawStrongText),
                cursorBrush = SolidColor(PawPrimary),
                decorationBox = {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        repeat(6) { index ->
                            val char = otpValue.getOrNull(index)?.toString() ?: ""
                            val isActive = otpValue.length == index

                            Column(
                                horizontalAlignment = Alignment.CenterHorizontally,
                            ) {
                                Text(
                                    text = char,
                                    style = MaterialTheme.typography.headlineLarge,
                                    fontSize = 32.sp,
                                    color = PawStrongText,
                                    textAlign = TextAlign.Center,
                                    modifier = Modifier.width(32.dp),
                                )
                                Spacer(Modifier.height(8.dp))
                                Box(
                                    modifier = Modifier
                                        .width(32.dp)
                                        .height(2.dp)
                                        .background(if (isActive) PawPrimary else PawOutline),
                                )
                            }
                        }
                    }
                },
            )
        }

        Spacer(Modifier.height(64.dp))

        // RETURN button
        Text(
            "RETURN",
            modifier = Modifier
                .clickable { navController.popBackStack() }
                .padding(horizontal = 24.dp, vertical = 12.dp),
            style = MaterialTheme.typography.labelLarge,
            color = PawMutedText,
        )

        Spacer(Modifier.weight(1f))
    }
}
