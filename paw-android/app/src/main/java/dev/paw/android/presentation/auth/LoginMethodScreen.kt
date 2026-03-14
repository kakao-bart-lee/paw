package dev.paw.android.presentation.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Email
import androidx.compose.material.icons.filled.Phone
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawSecure
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface2
import dev.paw.android.presentation.theme.PawAmber

@Composable
fun LoginMethodScreen(navController: NavController) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(PawBackground)
            .statusBarsPadding(),
    ) {
        // Header
        Row(
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(
                onClick = { navController.popBackStack() },
                modifier = Modifier
                    .size(40.dp)
                    .background(PawSurface2, CircleShape),
            ) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
        }

        // Content
        Column(
            modifier = Modifier
                .weight(1f)
                .padding(horizontal = 24.dp),
        ) {
            Text(
                "로그인 방법 선택",
                style = MaterialTheme.typography.headlineMedium,
                color = PawStrongText,
            )
            Text(
                "계정에 연결할 방법을 선택해주세요",
                style = MaterialTheme.typography.bodyMedium,
                color = PawMutedText,
                modifier = Modifier.padding(top = 8.dp, bottom = 32.dp),
            )

            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                // Phone option
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(PawSurface1, RoundedCornerShape(16.dp))
                        .border(1.dp, PawOutline, RoundedCornerShape(16.dp))
                        .clickable { navController.navigate(PawRoutes.PHONE_INPUT) }
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(16.dp),
                ) {
                    Box(
                        modifier = Modifier
                            .size(48.dp)
                            .background(PawAmber.copy(alpha = 0.1f), RoundedCornerShape(12.dp)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(Icons.Filled.Phone, null, tint = PawAmber, modifier = Modifier.size(24.dp))
                    }
                    Column(modifier = Modifier.weight(1f)) {
                        Text("전화번호로 계속", style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                        Text("SMS로 인증번호를 받습니다", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                    }
                }

                // Email option (disabled)
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .alpha(0.5f)
                        .background(PawSurface1, RoundedCornerShape(16.dp))
                        .border(1.dp, PawOutline, RoundedCornerShape(16.dp))
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(16.dp),
                ) {
                    Box(
                        modifier = Modifier
                            .size(48.dp)
                            .background(PawMutedText.copy(alpha = 0.1f), RoundedCornerShape(12.dp)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(Icons.Filled.Email, null, tint = PawMutedText, modifier = Modifier.size(24.dp))
                    }
                    Column(modifier = Modifier.weight(1f)) {
                        Text("이메일로 계속", style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                        Text("곧 지원 예정", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                    }
                }
            }

            // Security note
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 32.dp)
                    .background(PawSecure.copy(alpha = 0.1f), RoundedCornerShape(16.dp))
                    .padding(16.dp),
            ) {
                Text(
                    "🔒 Paw는 종단간 암호화를 사용하여 모든 대화를 안전하게 보호합니다.",
                    style = MaterialTheme.typography.bodySmall,
                    color = PawSecure,
                )
            }
        }
    }
}
