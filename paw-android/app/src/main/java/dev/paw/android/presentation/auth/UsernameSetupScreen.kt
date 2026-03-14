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
import androidx.compose.material.icons.filled.AlternateEmail
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
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
fun UsernameSetupScreen(navController: NavController, viewModel: BootstrapViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val authVm = viewModel.authViewModel
    val isLoading = uiState.preview.auth.isLoading
    val error = uiState.preview.auth.error
    val username = uiState.usernameInput

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
            Text("사용자 이름 설정", style = MaterialTheme.typography.headlineMedium, color = PawStrongText)
            Text(
                "다른 사람들이 나를 찾을 수 있는 고유한 이름을 만들어주세요",
                style = MaterialTheme.typography.bodyMedium,
                color = PawMutedText,
                modifier = Modifier.padding(top = 8.dp, bottom = 32.dp),
            )

            Text("사용자 이름", style = MaterialTheme.typography.labelLarge, color = PawStrongText, modifier = Modifier.padding(bottom = 8.dp))

            TextField(
                value = username,
                onValueChange = authVm::onUsernameChanged,
                modifier = Modifier.fillMaxWidth().height(56.dp),
                placeholder = { Text("username") },
                leadingIcon = { Icon(Icons.Filled.AlternateEmail, null, tint = PawMutedText) },
                trailingIcon = {
                    if (username.length >= 3) {
                        val isReserved = username.lowercase() in listOf("admin", "paw")
                        Icon(
                            if (isReserved) Icons.Filled.Close else Icons.Filled.Check,
                            null,
                            tint = if (isReserved) PawDestructive else PawSecure,
                        )
                    }
                },
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

            if (username.isNotEmpty() && username.length < 3) {
                Text("최소 3자 이상 입력해주세요", style = MaterialTheme.typography.bodySmall, color = PawMutedText, modifier = Modifier.padding(top = 8.dp))
            }

            // Discoverable toggle
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 24.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("전화번호로 검색 허용", style = MaterialTheme.typography.bodyMedium, color = PawStrongText)
                Switch(
                    checked = uiState.discoverableByPhone,
                    onCheckedChange = authVm::onDiscoverableChanged,
                    colors = SwitchDefaults.colors(checkedTrackColor = PawPrimary),
                )
            }

            Spacer(Modifier.height(24.dp))

            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("• 영문 소문자, 숫자, 밑줄(_)만 사용 가능", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                Text("• 3-20자 사이로 설정해주세요", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
            }

            if (error != null) {
                Text(error, style = MaterialTheme.typography.bodySmall, color = PawDestructive, modifier = Modifier.padding(top = 8.dp))
            }
        }

        Column(
            modifier = Modifier.padding(horizontal = 24.dp, vertical = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Button(
                onClick = { authVm.completeUsernameSetup() },
                modifier = Modifier.fillMaxWidth().height(56.dp),
                enabled = username.length >= 3 && !isLoading,
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
                    Text("완료", style = MaterialTheme.typography.titleMedium)
                }
            }

            TextButton(
                onClick = { authVm.skipUsernameSetup() },
                modifier = Modifier.fillMaxWidth().height(56.dp),
                shape = RoundedCornerShape(16.dp),
            ) {
                Text("나중에 설정하기", style = MaterialTheme.typography.titleMedium, color = PawMutedText)
            }
        }
    }
}
