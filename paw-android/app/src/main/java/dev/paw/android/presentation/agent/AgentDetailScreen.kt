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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import dev.paw.android.presentation.theme.PawAI
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawStrongText

// Re-use the signals list from AgentHubScreen
private val signals = listOf(
    SignalEntity("1", "Oracle", "ANALYSIS", "Sees patterns in chaos", listOf("PATTERN RECOGNITION", "PREDICTION", "SYNTHESIS"), bound = true),
    SignalEntity("2", "Scribe", "CREATION", "Transforms thought to word", listOf("WRITING", "TRANSLATION", "SUMMARIZATION"), bound = true),
    SignalEntity("3", "Sentinel", "SECURITY", "Guards the threshold", listOf("ENCRYPTION", "VERIFICATION", "PROTECTION"), bound = false),
    SignalEntity("4", "Weaver", "INTEGRATION", "Connects disparate threads", listOf("AUTOMATION", "LINKING"), bound = false),
)

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun AgentDetailScreen(agentId: String, navController: NavController) {
    val signal = signals.firstOrNull { it.id == agentId } ?: return
    var bound by remember { mutableStateOf(signal.bound) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(PawBackground)
            .statusBarsPadding()
            .verticalScroll(rememberScrollState()),
    ) {
        // Header
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = { navController.popBackStack() }) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
        }

        // Signal info
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp, vertical = 16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            // Bound indicator
            Box(
                modifier = Modifier
                    .size(64.dp)
                    .border(1.dp, if (bound) PawAI else PawOutline, CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                if (bound) {
                    Icon(Icons.Filled.Check, null, tint = PawAI, modifier = Modifier.size(24.dp))
                }
            }

            Spacer(Modifier.height(24.dp))

            Row(
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(signal.name, style = MaterialTheme.typography.titleLarge, color = PawStrongText)
                Text(signal.domain, style = MaterialTheme.typography.labelSmall, color = PawMutedText)
            }

            Spacer(Modifier.height(8.dp))

            Text(
                signal.essence,
                style = MaterialTheme.typography.bodyMedium.copy(fontStyle = FontStyle.Italic),
                color = PawMutedText,
                textAlign = TextAlign.Center,
            )

            Spacer(Modifier.height(24.dp))

            // Abilities
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                signal.abilities.forEach { ability ->
                    Text(
                        ability,
                        modifier = Modifier
                            .border(1.dp, PawOutline, RoundedCornerShape(4.dp))
                            .padding(horizontal = 12.dp, vertical = 8.dp),
                        style = MaterialTheme.typography.labelSmall,
                        color = PawMutedText,
                    )
                }
            }

            Spacer(Modifier.height(48.dp))

            // Bind/Unbind action
            Text(
                if (bound) "UNBIND" else "BIND",
                modifier = Modifier
                    .clickable { bound = !bound }
                    .border(1.dp, if (bound) PawOutline else PawAI, RoundedCornerShape(4.dp))
                    .padding(horizontal = 32.dp, vertical = 12.dp),
                style = MaterialTheme.typography.labelLarge,
                color = if (bound) PawMutedText else PawAI,
            )
        }
    }
}
