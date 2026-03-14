package dev.paw.android.presentation.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
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
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawPrimaryForeground
import dev.paw.android.presentation.theme.PawSecure
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface3

private data class Contact(val id: String, val name: String, val username: String)

private val contacts = listOf(
    Contact("1", "김민지", "minji_kim"),
    Contact("2", "박서준", "seojun_park"),
    Contact("3", "이수진", "sujin_lee"),
    Contact("4", "정우성", "woosung_j"),
    Contact("5", "최지원", "jiwon_choi"),
)

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun GroupCreateScreen(navController: NavController) {
    var step by remember { mutableStateOf("select") }
    val selectedMembers = remember { mutableStateListOf<String>() }
    var groupName by remember { mutableStateOf("") }
    var isEncrypted by remember { mutableStateOf(true) }

    Column(
        modifier = Modifier.fillMaxSize().background(PawBackground).statusBarsPadding(),
    ) {
        // Header
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = {
                if (step == "setup") step = "select" else navController.popBackStack()
            }) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
            Text(
                if (step == "select") "멤버 선택" else "그룹 설정",
                style = MaterialTheme.typography.titleLarge,
                color = PawStrongText,
                modifier = Modifier.weight(1f),
            )
            Button(
                onClick = {
                    if (step == "select" && selectedMembers.isNotEmpty()) step = "setup"
                    else if (step == "setup" && groupName.isNotBlank()) navController.navigate(PawRoutes.CHAT_LIST) {
                        popUpTo(PawRoutes.CHAT_LIST) { inclusive = true }
                    }
                },
                enabled = if (step == "select") selectedMembers.isNotEmpty() else groupName.isNotBlank(),
                shape = RoundedCornerShape(999.dp),
                colors = ButtonDefaults.buttonColors(containerColor = PawPrimary, contentColor = PawPrimaryForeground),
            ) {
                Text(if (step == "select") "다음" else "만들기")
            }
        }

        if (step == "select") {
            LazyColumn(modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
                item {
                    Text("연락처", style = MaterialTheme.typography.labelSmall, color = PawMutedText, modifier = Modifier.padding(vertical = 8.dp))
                }
                items(contacts) { contact ->
                    val selected = contact.id in selectedMembers
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable {
                                if (selected) selectedMembers.remove(contact.id) else selectedMembers.add(contact.id)
                            }
                            .padding(vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Box(
                            modifier = Modifier.size(48.dp).background(
                                if (selected) PawPrimary else PawSurface3,
                                CircleShape,
                            ),
                            contentAlignment = Alignment.Center,
                        ) {
                            if (selected) {
                                Icon(Icons.Filled.Check, null, tint = PawPrimaryForeground, modifier = Modifier.size(20.dp))
                            } else {
                                Text(contact.name.first().toString(), style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                            }
                        }
                        Column(modifier = Modifier.weight(1f)) {
                            Text(contact.name, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                            Text("@${contact.username}", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                        }
                    }
                }
            }
        } else {
            Column(modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
                // Group name
                TextField(
                    value = groupName,
                    onValueChange = { groupName = it },
                    modifier = Modifier.fillMaxWidth().padding(vertical = 16.dp),
                    placeholder = { Text("그룹 이름 입력") },
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
                    textStyle = MaterialTheme.typography.titleMedium,
                )

                // Encryption toggle
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(PawSurface1, RoundedCornerShape(16.dp))
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Box(
                        modifier = Modifier.size(40.dp).background(PawSecure.copy(alpha = 0.1f), RoundedCornerShape(12.dp)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(Icons.Filled.Lock, null, tint = PawSecure, modifier = Modifier.size(20.dp))
                    }
                    Column(modifier = Modifier.weight(1f).padding(start = 12.dp)) {
                        Text("종단간 암호화", style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                        Text("모든 메시지가 암호화됩니다", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                    }
                    Switch(checked = isEncrypted, onCheckedChange = { isEncrypted = it }, colors = SwitchDefaults.colors(checkedTrackColor = PawPrimary))
                }

                // Members summary
                Text(
                    "참여자 (${selectedMembers.size + 1}명)",
                    style = MaterialTheme.typography.labelSmall,
                    color = PawMutedText,
                    modifier = Modifier.padding(top = 24.dp, bottom = 8.dp),
                )
                FlowRow(
                    modifier = Modifier.fillMaxWidth().background(PawSurface1, RoundedCornerShape(16.dp)).padding(16.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Box(
                        modifier = Modifier.background(PawPrimary.copy(alpha = 0.1f), RoundedCornerShape(999.dp)).padding(horizontal = 12.dp, vertical = 6.dp),
                    ) {
                        Text("나", style = MaterialTheme.typography.bodySmall, color = PawPrimary)
                    }
                    selectedMembers.forEach { id ->
                        val contact = contacts.firstOrNull { it.id == id }
                        if (contact != null) {
                            Box(
                                modifier = Modifier.background(PawSurface3, RoundedCornerShape(999.dp)).padding(horizontal = 12.dp, vertical = 6.dp),
                            ) {
                                Text(contact.name, style = MaterialTheme.typography.bodySmall, color = PawStrongText)
                            }
                        }
                    }
                }
            }
        }
    }
}
