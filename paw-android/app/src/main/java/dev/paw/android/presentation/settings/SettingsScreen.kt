package dev.paw.android.presentation.settings

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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.components.PawBottomNavBar
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawAI
import dev.paw.android.presentation.theme.PawAmber
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawSecure
import dev.paw.android.presentation.theme.PawStrongText

@Composable
fun SettingsScreen(navController: NavController, viewModel: BootstrapViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val username = uiState.preview.auth.username
    val phone = uiState.preview.auth.phone

    Scaffold(
        containerColor = PawBackground,
        bottomBar = { PawBottomNavBar(currentRoute = PawRoutes.SETTINGS, navController = navController) },
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .statusBarsPadding()
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            // Header
            Text(
                "SELF",
                style = MaterialTheme.typography.labelLarge,
                color = PawMutedText,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 24.dp),
            )

            Spacer(Modifier.height(16.dp))

            // Avatar with concentric rings
            val ringColor1 = PawPrimary
            val ringColor2 = PawPrimary.copy(alpha = 0.4f)
            val ringColor3 = PawPrimary.copy(alpha = 0.15f)

            Box(
                modifier = Modifier
                    .size(120.dp)
                    .drawBehind {
                        // Outer ring
                        drawCircle(
                            color = ringColor3,
                            radius = size.minDimension / 2f,
                            style = Stroke(width = 1.dp.toPx()),
                        )
                        // Middle ring
                        drawCircle(
                            color = ringColor2,
                            radius = size.minDimension / 2f * 0.8f,
                            style = Stroke(width = 1.dp.toPx()),
                        )
                        // Inner filled
                        drawCircle(
                            color = ringColor1.copy(alpha = 0.1f),
                            radius = size.minDimension / 2f * 0.6f,
                        )
                    },
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    (username.firstOrNull() ?: 'U').uppercase(),
                    style = MaterialTheme.typography.headlineMedium,
                    color = PawPrimary,
                )
            }

            Spacer(Modifier.height(16.dp))

            Text(
                username.ifBlank { "User" },
                style = MaterialTheme.typography.titleLarge,
                color = PawStrongText,
            )

            Text(
                phone.ifBlank { "+82 10-****-5678" },
                style = MaterialTheme.typography.bodySmall,
                color = PawMutedText,
                modifier = Modifier.padding(top = 4.dp),
            )

            Spacer(Modifier.height(32.dp))

            // Stats
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 64.dp),
                horizontalArrangement = Arrangement.SpaceEvenly,
            ) {
                StatColumn("3", "SIGNALS")
                StatColumn("5", "CONNECTIONS")
            }

            Spacer(Modifier.height(40.dp))

            // CONFIGURATION
            Column(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 24.dp),
                verticalArrangement = Arrangement.spacedBy(0.dp),
            ) {
                Text(
                    "CONFIGURATION",
                    style = MaterialTheme.typography.labelSmall,
                    color = PawMutedText,
                    modifier = Modifier.padding(bottom = 20.dp),
                )

                ConfigRow(Icons.Filled.Lock, "Encryption", "End-to-end", PawSecure)
                ConfigRow(Icons.Filled.Shield, "Security Key", "Verified", PawSecure)
                ConfigRow(Icons.Filled.Visibility, "Read Receipts", "Hidden", PawMutedText)
                ConfigRow(Icons.Filled.Notifications, "Notifications", "Selective", PawSecure)
            }

            Spacer(Modifier.height(32.dp))
        }
    }
}

@Composable
private fun StatColumn(value: String, label: String) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(
            value,
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Normal,
            color = PawStrongText,
        )
        Text(
            label,
            style = MaterialTheme.typography.labelSmall,
            color = PawMutedText,
        )
    }
}

@Composable
private fun ConfigRow(
    icon: ImageVector,
    label: String,
    value: String,
    dotColor: androidx.compose.ui.graphics.Color,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            icon,
            null,
            tint = PawMutedText,
            modifier = Modifier.size(20.dp),
        )
        Text(
            label,
            style = MaterialTheme.typography.bodyMedium,
            color = PawStrongText,
            modifier = Modifier
                .weight(1f)
                .padding(start = 16.dp),
        )
        Text(
            value,
            style = MaterialTheme.typography.bodySmall,
            color = PawMutedText,
        )
        Box(
            modifier = Modifier
                .padding(start = 8.dp)
                .size(8.dp)
                .background(dotColor, CircleShape),
        )
    }
}
