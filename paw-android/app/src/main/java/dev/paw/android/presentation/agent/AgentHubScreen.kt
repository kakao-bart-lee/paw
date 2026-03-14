package dev.paw.android.presentation.agent

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import dev.paw.android.presentation.components.PawBottomNavBar
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawAI
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawStrongText

data class SignalEntity(
    val id: String,
    val name: String,
    val domain: String,
    val essence: String,
    val abilities: List<String>,
    val bound: Boolean,
)

private val signals = listOf(
    SignalEntity("1", "Oracle", "ANALYSIS", "Sees patterns in chaos", listOf("PATTERN RECOGNITION", "PREDICTION", "SYNTHESIS"), bound = true),
    SignalEntity("2", "Scribe", "CREATION", "Transforms thought to word", listOf("WRITING", "TRANSLATION", "SUMMARIZATION"), bound = true),
    SignalEntity("3", "Sentinel", "SECURITY", "Guards the threshold", listOf("ENCRYPTION", "VERIFICATION", "PROTECTION"), bound = false),
    SignalEntity("4", "Weaver", "INTEGRATION", "Connects disparate threads", listOf("AUTOMATION", "LINKING"), bound = false),
)

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun AgentHubScreen(navController: NavController) {
    Scaffold(
        containerColor = PawBackground,
        bottomBar = { PawBottomNavBar(currentRoute = PawRoutes.AGENT_HUB, navController = navController) },
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .statusBarsPadding(),
        ) {
            // Header
            Column(
                modifier = Modifier.padding(horizontal = 24.dp, vertical = 24.dp),
            ) {
                Text(
                    "SIGNALS",
                    style = MaterialTheme.typography.labelLarge,
                    color = PawStrongText,
                )
                Spacer(Modifier.height(8.dp))
                Text(
                    "AI entities that can be bound to your presence",
                    style = MaterialTheme.typography.bodySmall,
                    color = PawMutedText,
                )
            }

            // Signal list
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.spacedBy(0.dp),
            ) {
                items(signals) { signal ->
                    SignalCard(
                        signal = signal,
                        onClick = { navController.navigate(PawRoutes.agentDetail(signal.id)) },
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun SignalCard(signal: SignalEntity, onClick: () -> Unit) {
    // Left border line
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
    ) {
        Box(
            modifier = Modifier
                .padding(start = 24.dp)
                .size(1.dp, 160.dp)
                .background(PawOutline),
        )

        Column(
            modifier = Modifier
                .weight(1f)
                .padding(start = 24.dp, end = 24.dp, top = 24.dp, bottom = 24.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        signal.name,
                        style = MaterialTheme.typography.titleMedium,
                        color = PawStrongText,
                    )
                    Text(
                        signal.domain,
                        style = MaterialTheme.typography.labelSmall,
                        color = PawMutedText,
                    )
                }

                // Bound indicator
                Box(
                    modifier = Modifier
                        .size(28.dp)
                        .border(
                            width = 1.dp,
                            color = if (signal.bound) PawAI else PawOutline,
                            shape = CircleShape,
                        )
                        .then(if (!signal.bound) Modifier.alpha(0.3f) else Modifier),
                    contentAlignment = Alignment.Center,
                ) {
                    if (signal.bound) {
                        Icon(
                            Icons.Filled.Check,
                            null,
                            tint = PawAI,
                            modifier = Modifier.size(14.dp),
                        )
                    }
                }
            }

            Spacer(Modifier.height(4.dp))

            Text(
                signal.essence,
                style = MaterialTheme.typography.bodySmall.copy(fontStyle = FontStyle.Italic),
                color = PawMutedText,
            )

            Spacer(Modifier.height(12.dp))

            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                signal.abilities.forEach { ability ->
                    Text(
                        ability,
                        modifier = Modifier
                            .border(1.dp, PawOutline, RoundedCornerShape(4.dp))
                            .padding(horizontal = 10.dp, vertical = 6.dp),
                        style = MaterialTheme.typography.labelSmall,
                        color = PawMutedText,
                    )
                }
            }
        }
    }

    // Separator
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 24.dp)
            .height(0.5.dp)
            .background(PawOutline),
    )
}
