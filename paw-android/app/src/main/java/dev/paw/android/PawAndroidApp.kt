package dev.paw.android

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.unit.dp
import dev.paw.android.ui.theme.PawAgentBubble
import dev.paw.android.ui.theme.PawAndroidTheme
import dev.paw.android.ui.theme.PawBackground
import dev.paw.android.ui.theme.PawMutedText
import dev.paw.android.ui.theme.PawOutline
import dev.paw.android.ui.theme.PawPrimary
import dev.paw.android.ui.theme.PawPrimarySoft
import dev.paw.android.ui.theme.PawReceivedBubble
import dev.paw.android.ui.theme.PawSentBubble
import dev.paw.android.ui.theme.PawStrongText
import dev.paw.android.ui.theme.PawSurface1

@Composable
fun PawAndroidApp() {
    PawAndroidTheme {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
            val coreStatus = remember { PawCoreBridge.describePing() }

            Scaffold(
                containerColor = PawBackground,
            ) { innerPadding ->
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(
                            brush = Brush.verticalGradient(
                                colors = listOf(PawSurface1, PawBackground),
                            ),
                        )
                        .padding(innerPadding)
                        .padding(24.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp),
                ) {
                    Text(
                        text = "Paw Android",
                        style = MaterialTheme.typography.headlineMedium,
                        color = PawStrongText,
                    )
                    Text(
                        text = "Flutter 버전의 다크 메신저 무드와 AI-first 분위기를 Android shell로 옮기는 기준 화면입니다.",
                        style = MaterialTheme.typography.bodyLarge,
                        color = PawMutedText,
                    )

                    MoodCard(
                        title = "대화 목록",
                        subtitle = "rounded cards · thin outline · layered surfaces",
                        background = PawReceivedBubble,
                    )

                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        MoodCard(
                            title = "보낸 메시지",
                            subtitle = "primary 강조",
                            background = PawSentBubble,
                            modifier = Modifier.weight(1f),
                        )
                        MoodCard(
                            title = "AI 스트림",
                            subtitle = "agent bubble 톤",
                            background = PawAgentBubble,
                            modifier = Modifier.weight(1f),
                        )
                    }

                    Text(
                        text = "paw-core status: $coreStatus",
                        style = MaterialTheme.typography.bodyMedium,
                        color = PawStrongText,
                    )
                }
            }
        }
    }
}

@Composable
private fun MoodCard(
    title: String,
    subtitle: String,
    background: androidx.compose.ui.graphics.Color,
    modifier: Modifier = Modifier,
) {
    Card(
        modifier = modifier
            .fillMaxWidth()
            .border(1.dp, PawOutline, RoundedCornerShape(22.dp)),
        shape = RoundedCornerShape(22.dp),
        colors = CardDefaults.cardColors(containerColor = background),
    ) {
        Column(
            modifier = Modifier.padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
            Text(subtitle, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
        }
    }
}
