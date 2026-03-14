package dev.paw.android.presentation.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.filled.Smartphone
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawDestructive
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawPrimaryForeground
import dev.paw.android.presentation.theme.PawSecure
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface2
import uniffi.paw_core.AuthStepView

@Composable
fun DeviceRegisterScreen(navController: NavController, viewModel: BootstrapViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val authVm = viewModel.authViewModel
    val isLoading = uiState.preview.auth.isLoading
    val error = uiState.preview.auth.error

    // Navigation is handled centrally by PawNavGraph's LaunchedEffect on authStep.

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(PawBackground)
            .statusBarsPadding(),
    ) {
        Row(modifier = Modifier.padding(horizontal = 16.dp, vertical = 16.dp)) {
            IconButton(
                onClick = { navController.popBackStack() },
                modifier = Modifier.size(40.dp).background(PawSurface2, CircleShape),
            ) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
        }

        Column(modifier = Modifier.weight(1f).padding(horizontal = 24.dp)) {
            Text("디바이스 등록", style = MaterialTheme.typography.headlineMedium, color = PawStrongText)
            Text(
                "이 기기의 이름을 설정해주세요",
                style = MaterialTheme.typography.bodyMedium,
                color = PawMutedText,
                modifier = Modifier.padding(top = 8.dp, bottom = 32.dp),
            )

            // Device icon
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 32.dp),
                contentAlignment = Alignment.Center,
            ) {
                Box(
                    modifier = Modifier
                        .size(96.dp)
                        .background(PawSurface2, RoundedCornerShape(24.dp)),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        Icons.Filled.Smartphone,
                        null,
                        tint = PawPrimary,
                        modifier = Modifier.size(48.dp),
                    )
                }
            }

            Text(
                "기기 이름",
                style = MaterialTheme.typography.labelLarge,
                color = PawStrongText,
                modifier = Modifier.padding(bottom = 8.dp),
            )
            TextField(
                value = uiState.deviceNameInput,
                onValueChange = authVm::onDeviceNameChanged,
                modifier = Modifier.fillMaxWidth().height(56.dp),
                placeholder = { Text("내 Android") },
                singleLine = true,
                shape = RoundedCornerShape(12.dp),
                colors = TextFieldDefaults.colors(
                    focusedContainerColor = PawSurface1,
                    unfocusedContainerColor = PawSurface1,
                    focusedTextColor = PawStrongText,
                    unfocusedTextColor = PawStrongText,
                    focusedIndicatorColor = Color.Transparent,
                    unfocusedIndicatorColor = Color.Transparent,
                    cursorColor = PawPrimary,
                ),
                textStyle = MaterialTheme.typography.bodyLarge,
            )

            // Info card
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 24.dp)
                    .background(PawSurface1, RoundedCornerShape(16.dp))
                    .padding(16.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Icon(Icons.Filled.Shield, null, tint = PawSecure, modifier = Modifier.size(20.dp))
                Text(
                    "기기 이름은 보안 설정에서 세션을 관리할 때 사용됩니다. 다른 사용자에게는 표시되지 않습니다.",
                    style = MaterialTheme.typography.bodySmall,
                    color = PawMutedText,
                )
            }

            if (error != null) {
                Text(error, style = MaterialTheme.typography.bodySmall, color = PawDestructive, modifier = Modifier.padding(top = 8.dp))
            }
        }

        Column(modifier = Modifier.padding(horizontal = 24.dp, vertical = 24.dp)) {
            Button(
                onClick = { authVm.registerDevice() },
                modifier = Modifier.fillMaxWidth().height(56.dp),
                enabled = uiState.deviceNameInput.isNotBlank() && !isLoading,
                shape = RoundedCornerShape(16.dp),
                colors = ButtonDefaults.buttonColors(
                    containerColor = PawPrimary,
                    contentColor = PawPrimaryForeground,
                    disabledContainerColor = PawPrimary.copy(alpha = 0.5f),
                ),
            ) {
                if (isLoading) {
                    CircularProgressIndicator(modifier = Modifier.size(24.dp), color = PawPrimaryForeground, strokeWidth = 2.dp)
                } else {
                    Text("계속", style = MaterialTheme.typography.titleMedium)
                }
            }
        }
    }
}
