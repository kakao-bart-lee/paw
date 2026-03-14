package dev.paw.android.presentation.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawStrongText

private data class NavTab(val route: String, val label: String)

private val tabs = listOf(
    NavTab(PawRoutes.CHAT_LIST, "STREAM"),
    NavTab(PawRoutes.AGENT_HUB, "SIGNALS"),
    NavTab(PawRoutes.SETTINGS, "SELF"),
)

@Composable
fun PawBottomNavBar(
    currentRoute: String?,
    navController: NavController,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .navigationBarsPadding()
            .padding(horizontal = 24.dp, vertical = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        tabs.forEach { tab ->
            val selected = currentRoute == tab.route
            Text(
                text = tab.label,
                modifier = Modifier
                    .weight(1f)
                    .clickable {
                        if (!selected) {
                            navController.navigate(tab.route) {
                                popUpTo(PawRoutes.CHAT_LIST) { saveState = true }
                                launchSingleTop = true
                                restoreState = true
                            }
                        }
                    }
                    .padding(vertical = 12.dp),
                style = MaterialTheme.typography.labelLarge,
                fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
                color = if (selected) PawStrongText else PawMutedText,
                textAlign = TextAlign.Center,
            )
        }
    }
}
