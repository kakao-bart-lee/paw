package dev.paw.android.presentation.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.KeyboardType
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
import dev.paw.android.presentation.theme.PawPrimaryForeground
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface2
import dev.paw.android.presentation.theme.PawDestructive

@Composable
fun PhoneInputScreen(navController: NavController, viewModel: BootstrapViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val authVm = viewModel.authViewModel
    val isLoading = uiState.preview.auth.isLoading
    val error = uiState.preview.auth.error

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(PawBackground)
            .statusBarsPadding(),
    ) {
        // Header
        Row(modifier = Modifier.padding(horizontal = 16.dp, vertical = 16.dp)) {
            IconButton(
                onClick = { navController.popBackStack() },
                modifier = Modifier.size(40.dp).background(PawSurface2, CircleShape),
            ) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
        }

        // Content
        Column(
            modifier = Modifier.weight(1f).padding(horizontal = 24.dp),
        ) {
            Text("전화번호 입력", style = MaterialTheme.typography.headlineMedium, color = PawStrongText)
            Text(
                "인증번호를 받을 전화번호를 입력해주세요",
                style = MaterialTheme.typography.bodyMedium,
                color = PawMutedText,
                modifier = Modifier.padding(top = 8.dp, bottom = 32.dp),
            )

            Row(
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                // Country code button
                Row(
                    modifier = Modifier
                        .height(56.dp)
                        .background(PawSurface1, RoundedCornerShape(12.dp))
                        .border(1.dp, PawOutline, RoundedCornerShape(12.dp))
                        .padding(horizontal = 16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text("🇰🇷", style = MaterialTheme.typography.bodyLarge)
                    Text("+82", style = MaterialTheme.typography.bodyLarge, color = PawStrongText)
                    Icon(Icons.Filled.KeyboardArrowDown, null, tint = PawMutedText, modifier = Modifier.size(16.dp))
                }

                // Phone input
                TextField(
                    value = uiState.phoneInput,
                    onValueChange = authVm::onPhoneChanged,
                    modifier = Modifier
                        .weight(1f)
                        .height(56.dp),
                    placeholder = { Text("010-1234-5678") },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone),
                    singleLine = true,
                    shape = RoundedCornerShape(12.dp),
                    colors = TextFieldDefaults.colors(
                        focusedContainerColor = PawSurface1,
                        unfocusedContainerColor = PawSurface1,
                        focusedTextColor = PawStrongText,
                        unfocusedTextColor = PawStrongText,
                        focusedIndicatorColor = Color.Transparent,
                        unfocusedIndicatorColor = Color.Transparent,
                        focusedPlaceholderColor = PawMutedText,
                        unfocusedPlaceholderColor = PawMutedText,
                        cursorColor = PawPrimary,
                    ),
                    textStyle = MaterialTheme.typography.bodyLarge,
                )
            }

            if (error != null) {
                Text(
                    error,
                    style = MaterialTheme.typography.bodySmall,
                    color = PawDestructive,
                    modifier = Modifier.padding(top = 8.dp),
                )
            } else {
                Text(
                    "표준 SMS 요금이 적용될 수 있습니다",
                    style = MaterialTheme.typography.bodySmall,
                    color = PawMutedText,
                    modifier = Modifier.padding(top = 16.dp),
                )
            }
        }

        // CTA
        Column(modifier = Modifier.padding(horizontal = 24.dp, vertical = 24.dp)) {
            Button(
                onClick = {
                    if (BuildConfig.DEBUG && uiState.phoneInput.isBlank()) {
                        authVm.devQuickLogin()
                    } else {
                        authVm.requestOtp()
                        navController.navigate(PawRoutes.OTP_VERIFY)
                    }
                },
                modifier = Modifier.fillMaxWidth().height(56.dp),
                enabled = !isLoading,
                shape = RoundedCornerShape(16.dp),
                colors = ButtonDefaults.buttonColors(
                    containerColor = PawPrimary,
                    contentColor = PawPrimaryForeground,
                    disabledContainerColor = PawPrimary.copy(alpha = 0.5f),
                ),
            ) {
                if (isLoading) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(24.dp),
                        color = PawPrimaryForeground,
                        strokeWidth = 2.dp,
                    )
                } else {
                    Text("인증번호 받기", style = MaterialTheme.typography.titleMedium)
                }
            }
        }
    }
}
