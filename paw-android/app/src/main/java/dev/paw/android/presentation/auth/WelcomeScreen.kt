package dev.paw.android.presentation.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dev.paw.android.BuildConfig
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawStrongText

@Composable
fun WelcomeScreen(navController: NavController, viewModel: BootstrapViewModel? = null) {
    val authVm = viewModel?.authViewModel
    val uiState = viewModel?.uiState?.collectAsStateWithLifecycle()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(PawBackground)
            .statusBarsPadding(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Spacer(Modifier.weight(1f))

        // Title
        Text(
            "Paw",
            style = MaterialTheme.typography.headlineLarge,
            color = PawStrongText,
        )

        Spacer(Modifier.height(12.dp))

        Text(
            "SIGNAL YOUR PRESENCE",
            style = MaterialTheme.typography.labelLarge,
            color = PawMutedText,
        )

        Spacer(Modifier.height(64.dp))

        // Phone input
        val phoneValue = uiState?.value?.phoneInput ?: ""
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 48.dp),
        ) {
            BasicTextField(
                value = phoneValue,
                onValueChange = { authVm?.onPhoneChanged(it) },
                modifier = Modifier.fillMaxWidth(),
                textStyle = MaterialTheme.typography.bodyLarge.copy(
                    color = PawStrongText,
                    textAlign = TextAlign.Center,
                ),
                singleLine = true,
                cursorBrush = SolidColor(PawPrimary),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone),
                decorationBox = { innerTextField ->
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Box(
                            modifier = Modifier.fillMaxWidth(),
                            contentAlignment = Alignment.Center,
                        ) {
                            if (phoneValue.isEmpty()) {
                                Text(
                                    "phone number",
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = PawMutedText,
                                )
                            }
                            innerTextField()
                        }
                        Spacer(Modifier.height(8.dp))
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(1.dp)
                                .background(PawOutline),
                        )
                    }
                },
            )
        }

        Spacer(Modifier.height(48.dp))

        // TRANSMIT button
        Text(
            "TRANSMIT",
            modifier = Modifier
                .clickable {
                    if (BuildConfig.DEBUG && phoneValue.isBlank()) {
                        authVm?.devQuickLogin()
                    } else if (authVm != null) {
                        authVm.showPhoneOtp()
                        authVm.onPhoneChanged(phoneValue)
                        authVm.requestOtp()
                        navController.navigate(PawRoutes.OTP_VERIFY)
                    }
                }
                .padding(horizontal = 24.dp, vertical = 12.dp),
            style = MaterialTheme.typography.labelLarge,
            color = PawMutedText,
        )

        if (BuildConfig.DEBUG && viewModel != null) {
            Spacer(Modifier.height(32.dp))
            Text(
                "BYPASS",
                modifier = Modifier
                    .clickable { viewModel.authViewModel.devQuickLogin() }
                    .padding(horizontal = 24.dp, vertical = 12.dp),
                style = MaterialTheme.typography.labelSmall,
                color = PawMutedText.copy(alpha = 0.4f),
            )
        }

        Spacer(Modifier.weight(1f))
    }
}
