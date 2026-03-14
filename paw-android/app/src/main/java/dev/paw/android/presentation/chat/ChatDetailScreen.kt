package dev.paw.android.presentation.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dev.paw.android.domain.model.ChatMessage
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.theme.PawAI
import dev.paw.android.presentation.theme.PawAmber
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface2

@Composable
fun ChatDetailScreen(
    chatId: String,
    navController: NavController,
    viewModel: BootstrapViewModel,
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val chatVm = viewModel.chatViewModel
    val conversation = uiState.chat.conversations.firstOrNull { it.id == chatId }
    val listState = rememberLazyListState()

    LaunchedEffect(uiState.chat.messages.size) {
        if (uiState.chat.messages.isNotEmpty()) {
            listState.animateScrollToItem(uiState.chat.messages.lastIndex)
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(PawBackground),
    ) {
        // Header
        Row(
            modifier = Modifier
                .statusBarsPadding()
                .fillMaxWidth()
                .padding(horizontal = 8.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = { navController.popBackStack() }) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }

            Column(modifier = Modifier.weight(1f).padding(start = 4.dp)) {
                Text(
                    conversation?.name ?: "대화",
                    style = MaterialTheme.typography.titleMedium,
                    color = PawStrongText,
                )
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Icon(Icons.Filled.Lock, null, tint = PawAI, modifier = Modifier.size(12.dp))
                    Text(
                        "ENCRYPTED",
                        style = MaterialTheme.typography.labelSmall,
                        color = PawMutedText,
                    )
                }
            }

            // Online dot
            Box(
                modifier = Modifier
                    .padding(end = 16.dp)
                    .size(10.dp)
                    .background(PawAmber, androidx.compose.foundation.shape.CircleShape),
            )
        }

        // Separator
        Box(
            modifier = Modifier.fillMaxWidth().height(0.5.dp).background(PawOutline),
        )

        // Messages
        LazyColumn(
            modifier = Modifier.weight(1f).fillMaxWidth(),
            state = listState,
            contentPadding = PaddingValues(horizontal = 24.dp, vertical = 24.dp),
            verticalArrangement = Arrangement.spacedBy(24.dp),
        ) {
            if (uiState.chat.messages.isNotEmpty()) {
                items(uiState.chat.messages, key = { it.id }) { message ->
                    EchoMessage(message)
                }
            } else {
                // Mock messages matching the screenshot
                val mockMessages = listOf(
                    MockEcho("the presentation is ready", "self"),
                    MockEcho("perfect timing, I just finished reviewing", "other"),
                    MockEcho("shall we do a final walkthrough?", "self"),
                    MockEcho("yes, I noticed a few points we could strengthen", "other"),
                    MockEcho("based on the context, consider emphasizing the ROI metrics in slide 7", "signal"),
                )
                items(mockMessages) { msg ->
                    MockEchoMessage(msg)
                }
            }
        }

        // Composer
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(horizontal = 24.dp, vertical = 16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(modifier = Modifier.weight(1f)) {
                BasicTextField(
                    value = uiState.chat.messageDraft,
                    onValueChange = chatVm::onMessageDraftChanged,
                    textStyle = MaterialTheme.typography.bodyMedium.copy(color = PawStrongText),
                    singleLine = true,
                    cursorBrush = SolidColor(PawPrimary),
                    decorationBox = { inner ->
                        if (uiState.chat.messageDraft.isEmpty()) {
                            Text("speak ...", style = MaterialTheme.typography.bodyMedium, color = PawMutedText)
                        }
                        inner()
                    },
                )
            }

            Text(
                "SEND",
                modifier = Modifier
                    .clickable(enabled = uiState.chat.messageDraft.isNotBlank()) {
                        chatVm.sendMessage()
                    }
                    .padding(start = 24.dp),
                style = MaterialTheme.typography.labelLarge,
                color = if (uiState.chat.messageDraft.isNotBlank()) PawStrongText else PawMutedText,
            )
        }
    }
}

@Composable
private fun EchoMessage(message: ChatMessage) {
    val isMe = message.isMe
    val isAgent = message.isAgent

    if (isAgent) {
        // SIGNAL message — centered, italic, amber
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                "SIGNAL",
                style = MaterialTheme.typography.labelSmall,
                color = PawAmber,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                message.content,
                style = MaterialTheme.typography.bodyMedium.copy(fontStyle = FontStyle.Italic),
                color = PawAmber.copy(alpha = 0.7f),
                textAlign = TextAlign.Center,
            )
            Spacer(Modifier.height(4.dp))
            Box(
                modifier = Modifier.size(48.dp, 2.dp).background(PawAmber.copy(alpha = 0.3f)),
            )
        }
    } else {
        // Regular message
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = if (isMe) Alignment.End else Alignment.Start,
        ) {
            Text(
                message.content,
                style = MaterialTheme.typography.bodyMedium,
                color = if (isMe) PawStrongText else PawMutedText,
                textAlign = if (isMe) TextAlign.End else TextAlign.Start,
            )
            Spacer(Modifier.height(4.dp))
            Box(
                modifier = Modifier
                    .fillMaxWidth(0.6f)
                    .height(0.5.dp)
                    .background(PawOutline)
                    .then(if (isMe) Modifier else Modifier),
            )
        }
    }
}

// Mock echo for when no real messages exist
private data class MockEcho(val text: String, val origin: String)

@Composable
private fun MockEchoMessage(msg: MockEcho) {
    if (msg.origin == "signal") {
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text("SIGNAL", style = MaterialTheme.typography.labelSmall, color = PawAmber)
            Spacer(Modifier.height(4.dp))
            Text(
                msg.text,
                style = MaterialTheme.typography.bodyMedium.copy(fontStyle = FontStyle.Italic),
                color = PawAmber.copy(alpha = 0.7f),
                textAlign = TextAlign.Center,
            )
            Spacer(Modifier.height(4.dp))
            Box(modifier = Modifier.size(48.dp, 2.dp).background(PawAmber.copy(alpha = 0.3f)))
        }
    } else {
        val isMe = msg.origin == "self"
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = if (isMe) Alignment.End else Alignment.Start,
        ) {
            Text(
                msg.text,
                style = MaterialTheme.typography.bodyMedium,
                color = if (isMe) PawStrongText else PawMutedText,
                textAlign = if (isMe) TextAlign.End else TextAlign.Start,
            )
            Spacer(Modifier.height(4.dp))
            Box(
                modifier = Modifier
                    .fillMaxWidth(0.6f)
                    .height(0.5.dp)
                    .background(PawOutline),
            )
        }
    }
}
