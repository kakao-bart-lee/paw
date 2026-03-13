package dev.paw.android

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.paw.android.ui.theme.PawAndroidTheme

@Composable
fun PawAndroidApp() {
    PawAndroidTheme {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
            val coreStatus = remember { PawCoreBridge.describePing() }

            Scaffold { innerPadding ->
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(innerPadding)
                        .padding(24.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Text(text = "Paw Android", style = MaterialTheme.typography.headlineMedium)
                    Text(
                        text = "Phase 1 native shell with Compose and a paw-core bridge hook.",
                        style = MaterialTheme.typography.bodyLarge,
                    )
                    Text(
                        text = "paw-core status: $coreStatus",
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}
