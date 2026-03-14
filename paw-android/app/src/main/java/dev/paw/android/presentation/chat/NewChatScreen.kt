package dev.paw.android.presentation.chat

import androidx.compose.foundation.background
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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Group
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.QrCode
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.SmartToy
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawAI
import dev.paw.android.presentation.theme.PawAmber
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawSecure
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface2
import dev.paw.android.presentation.theme.PawSurface3

private data class MockContact(val id: String, val name: String, val username: String, val isOnline: Boolean)

private val mockContacts = listOf(
    MockContact("1", "김민지", "minji_kim", true),
    MockContact("2", "박서준", "seojun_park", false),
    MockContact("3", "이수진", "sujin_lee", true),
    MockContact("4", "정우성", "woosung_j", false),
    MockContact("5", "최지원", "jiwon_choi", true),
)

@Composable
fun NewChatScreen(navController: NavController, viewModel: BootstrapViewModel) {
    var query by remember { mutableStateOf("") }
    val filtered = mockContacts.filter {
        it.name.contains(query, ignoreCase = true) || it.username.contains(query, ignoreCase = true)
    }

    Column(
        modifier = Modifier.fillMaxSize().background(PawBackground).statusBarsPadding(),
    ) {
        // Header
        Row(
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            IconButton(onClick = { navController.popBackStack() }) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
            Text("새 대화", style = MaterialTheme.typography.titleLarge, color = PawStrongText)
        }

        // Search
        TextField(
            value = query,
            onValueChange = { query = it },
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
            placeholder = { Text("이름 또는 사용자 이름 검색") },
            leadingIcon = { Icon(Icons.Filled.Search, null, tint = PawMutedText) },
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

        LazyColumn(modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
            // Quick actions
            item {
                Column(
                    modifier = Modifier.padding(vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    QuickAction(Icons.Filled.Group, "새 그룹", "여러 사람과 대화하기", PawAmber) {
                        navController.navigate(PawRoutes.GROUP_CREATE)
                    }
                    QuickAction(Icons.Filled.SmartToy, "Agent와 대화", "AI 어시스턴트 시작하기", PawAI) {
                        navController.navigate(PawRoutes.AGENT_HUB)
                    }
                    QuickAction(Icons.Filled.PersonAdd, "친구 초대", "연락처에서 초대하기", PawStrongText) {}
                    QuickAction(Icons.Filled.QrCode, "QR 코드", "QR로 친구 추가하기", PawStrongText) {}
                }
            }

            // Section header
            item {
                Text(
                    "연락처",
                    style = MaterialTheme.typography.labelSmall,
                    color = PawMutedText,
                    modifier = Modifier.padding(top = 16.dp, bottom = 8.dp),
                )
            }

            items(filtered) { contact ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { navController.navigate(PawRoutes.chatDetail(contact.id)) }
                        .padding(vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Box(
                        modifier = Modifier.size(48.dp).background(PawSurface3, CircleShape),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(contact.name.first().toString(), style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                    }
                    Column(modifier = Modifier.weight(1f)) {
                        Text(contact.name, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                        Text("@${contact.username}", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                    }
                    Text(
                        if (contact.isOnline) "온라인" else "오프라인",
                        style = MaterialTheme.typography.bodySmall,
                        color = if (contact.isOnline) PawSecure else PawMutedText,
                    )
                }
            }
        }
    }
}

@Composable
private fun QuickAction(
    icon: ImageVector,
    title: String,
    subtitle: String,
    tint: Color,
    onClick: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Box(
            modifier = Modifier.size(48.dp).background(tint.copy(alpha = 0.1f), CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Icon(icon, null, tint = tint, modifier = Modifier.size(24.dp))
        }
        Column {
            Text(title, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
            Text(subtitle, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
        }
    }
}
