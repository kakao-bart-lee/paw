package dev.paw.android.presentation.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.AlternateEmail
import androidx.compose.material.icons.filled.CameraAlt
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Phone
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawPrimaryForeground
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface3

@Composable
fun ProfileScreen(navController: NavController, viewModel: BootstrapViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val username = uiState.preview.auth.username
    val phone = uiState.preview.auth.phone
    var displayName by remember { mutableStateOf(username.ifBlank { "사용자" }) }
    var status by remember { mutableStateOf("안녕하세요! Paw를 사용 중입니다.") }
    var isEditing by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier.fillMaxSize().background(PawBackground).statusBarsPadding(),
    ) {
        // Header
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = { navController.popBackStack() }) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
            Text("프로필", style = MaterialTheme.typography.titleLarge, color = PawStrongText, modifier = Modifier.weight(1f))
            if (isEditing) {
                Button(
                    onClick = { isEditing = false },
                    shape = RoundedCornerShape(999.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = PawPrimary, contentColor = PawPrimaryForeground),
                ) {
                    Icon(Icons.Filled.Check, null, modifier = Modifier.size(16.dp))
                    Text("저장", modifier = Modifier.padding(start = 4.dp))
                }
            } else {
                TextButton(onClick = { isEditing = true }) {
                    Icon(Icons.Filled.Edit, null, tint = PawPrimary, modifier = Modifier.size(16.dp))
                    Text("편집", color = PawPrimary, modifier = Modifier.padding(start = 4.dp))
                }
            }
        }

        Column(
            modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()),
        ) {
            // Avatar
            Column(
                modifier = Modifier.fillMaxWidth().padding(vertical = 24.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Box {
                    Box(
                        modifier = Modifier.size(112.dp).background(PawPrimary.copy(alpha = 0.1f), CircleShape),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            displayName.firstOrNull()?.uppercase() ?: "U",
                            style = MaterialTheme.typography.headlineLarge,
                            color = PawPrimary,
                        )
                    }
                    if (isEditing) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.BottomEnd)
                                .size(40.dp)
                                .background(PawPrimary, CircleShape),
                            contentAlignment = Alignment.Center,
                        ) {
                            Icon(Icons.Filled.CameraAlt, null, tint = PawPrimaryForeground, modifier = Modifier.size(20.dp))
                        }
                    }
                }

                if (isEditing) {
                    TextField(
                        value = displayName,
                        onValueChange = { displayName = it },
                        modifier = Modifier.padding(top = 16.dp),
                        textStyle = MaterialTheme.typography.titleLarge.copy(textAlign = TextAlign.Center),
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
                    )
                } else {
                    Text(displayName, style = MaterialTheme.typography.titleLarge, color = PawStrongText, modifier = Modifier.padding(top = 16.dp))
                }

                if (username.isNotBlank()) {
                    Text("@$username", style = MaterialTheme.typography.bodySmall, color = PawMutedText, modifier = Modifier.padding(top = 4.dp))
                }
            }

            // Status
            Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
                Text("상태 메시지", style = MaterialTheme.typography.labelSmall, color = PawMutedText, modifier = Modifier.padding(bottom = 8.dp))
                if (isEditing) {
                    TextField(
                        value = status,
                        onValueChange = { status = it },
                        modifier = Modifier.fillMaxWidth(),
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
                    )
                } else {
                    Box(modifier = Modifier.fillMaxWidth().background(PawSurface1, RoundedCornerShape(16.dp)).padding(16.dp)) {
                        Text(status, style = MaterialTheme.typography.bodyMedium, color = PawStrongText)
                    }
                }
            }

            // Contact info
            Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
                Text("연락처 정보", style = MaterialTheme.typography.labelSmall, color = PawMutedText, modifier = Modifier.padding(bottom = 8.dp))
                Column(modifier = Modifier.background(PawSurface1, RoundedCornerShape(16.dp))) {
                    if (username.isNotBlank()) {
                        ContactInfoRow(Icons.Filled.AlternateEmail, "사용자 이름", "@$username")
                        Box(modifier = Modifier.fillMaxWidth().padding(start = 68.dp).size(0.5.dp).background(PawOutline))
                    }
                    ContactInfoRow(Icons.Filled.Phone, "전화번호", phone.ifBlank { "+82 10-****-****" })
                }
            }

            // Privacy note
            Box(
                modifier = Modifier.padding(16.dp).fillMaxWidth().background(PawSurface1, RoundedCornerShape(16.dp)).padding(16.dp),
            ) {
                Text(
                    "프로필 사진과 상태 메시지는 연락처에 저장된 사람들에게 표시됩니다. 개인정보 설정에서 공개 범위를 변경할 수 있습니다.",
                    style = MaterialTheme.typography.bodySmall,
                    color = PawMutedText,
                )
            }
        }
    }
}

@Composable
private fun ContactInfoRow(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    label: String,
    value: String,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Box(modifier = Modifier.size(40.dp).background(PawSurface3, RoundedCornerShape(12.dp)), contentAlignment = Alignment.Center) {
            Icon(icon, null, tint = PawStrongText, modifier = Modifier.size(20.dp))
        }
        Column {
            Text(label, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
            Text(value, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
        }
    }
}
