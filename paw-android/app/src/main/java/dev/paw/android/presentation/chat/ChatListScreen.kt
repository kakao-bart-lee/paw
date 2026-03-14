package dev.paw.android.presentation.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.components.PawBottomNavBar
import dev.paw.android.presentation.components.WaveformIcon
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawAI
import dev.paw.android.presentation.theme.PawAmber
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawStrongText

// Mock entities matching the design screenshots
private data class StreamEntity(
    val id: String,
    val name: String,
    val type: String,      // "human", "ai", "collective"
    val signature: String,  // "sine", "pulse", "wave", "fractal"
    val time: String,
    val fragment: String,
    val isSignal: Boolean = false,
)

private val mockEntities = listOf(
    StreamEntity("1", "Echo", "ai", "pulse", "active", "ready to assist", isSignal = true),
    StreamEntity("2", "Mina", "human", "sine", "now", "the meeting went better than expect…", isSignal = false),
    StreamEntity("3", "Project Void", "collective", "wave", "30m", "new designs uploaded", isSignal = false),
    StreamEntity("4", "Cipher", "ai", "fractal", "2h", "analysis complete", isSignal = true),
    StreamEntity("5", "Kai", "human", "sine", "yesterday", "weekend plans?", isSignal = false),
)

@Composable
fun ChatListScreen(navController: NavController, viewModel: BootstrapViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    Scaffold(
        containerColor = PawBackground,
        bottomBar = {
            PawBottomNavBar(currentRoute = PawRoutes.CHAT_LIST, navController = navController)
        },
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .statusBarsPadding(),
        ) {
            // Header
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 24.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    "STREAM",
                    style = MaterialTheme.typography.labelLarge,
                    color = PawMutedText,
                )
                IconButton(
                    onClick = { navController.navigate(PawRoutes.NEW_CHAT) },
                    modifier = Modifier.size(24.dp),
                ) {
                    Icon(Icons.Filled.Add, "새 대화", tint = PawMutedText)
                }
            }

            // Entity list with left border
            val entities = if (uiState.chat.conversations.isNotEmpty()) {
                // Map real conversations to stream entities
                uiState.chat.conversations.mapIndexed { index, conv ->
                    val mock = mockEntities.getOrNull(index)
                    StreamEntity(
                        id = conv.id,
                        name = conv.name,
                        type = mock?.type ?: "human",
                        signature = mock?.signature ?: "sine",
                        time = mock?.time ?: "",
                        fragment = conv.lastMessage ?: "최근 메시지 없음",
                        isSignal = mock?.isSignal ?: false,
                    )
                }
            } else {
                mockEntities
            }

            LazyColumn(
                modifier = Modifier.fillMaxSize(),
            ) {
                items(entities) { entity ->
                    EntityRow(
                        entity = entity,
                        onClick = {
                            if (uiState.chat.conversations.any { it.id == entity.id }) {
                                viewModel.chatViewModel.selectConversation(entity.id)
                            }
                            navController.navigate(PawRoutes.chatDetail(entity.id))
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun EntityRow(entity: StreamEntity, onClick: () -> Unit) {
    val waveColor = when (entity.type) {
        "ai" -> PawAI
        "collective" -> PawMutedText
        else -> PawAmber
    }
    val dotColor = when (entity.type) {
        "ai" -> PawAI
        else -> PawAmber
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // Left teal border line
        Box(
            modifier = Modifier
                .width(2.dp)
                .height(90.dp)
                .background(PawAI.copy(alpha = 0.3f)),
        )

        // Waveform icon
        WaveformIcon(
            signature = entity.signature,
            color = waveColor,
            modifier = Modifier
                .padding(start = 20.dp)
                .size(48.dp, 32.dp),
        )

        // Content
        Column(
            modifier = Modifier
                .weight(1f)
                .padding(start = 16.dp, top = 20.dp, bottom = 20.dp, end = 16.dp),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Text(
                    entity.name,
                    style = MaterialTheme.typography.titleMedium,
                    color = PawStrongText,
                )
                Text(
                    entity.time,
                    style = MaterialTheme.typography.bodySmall,
                    color = PawMutedText,
                )
                if (entity.isSignal) {
                    Text(
                        "SIGNAL",
                        style = MaterialTheme.typography.labelSmall,
                        color = PawAI,
                    )
                }
            }

            Text(
                entity.fragment,
                style = MaterialTheme.typography.bodySmall,
                color = PawMutedText,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 4.dp),
            )
        }

        // Status dot
        Box(
            modifier = Modifier
                .padding(end = 24.dp)
                .size(8.dp)
                .background(dotColor, androidx.compose.foundation.shape.CircleShape),
        )
    }

    // Separator
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 86.dp)
            .height(0.5.dp)
            .background(PawOutline),
    )
}
