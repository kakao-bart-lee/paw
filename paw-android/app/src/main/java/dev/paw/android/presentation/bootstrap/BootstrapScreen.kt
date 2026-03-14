package dev.paw.android.presentation.bootstrap

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dev.paw.android.presentation.navigation.PawNavGraph
import dev.paw.android.presentation.theme.PawAndroidTheme

@Composable
fun BootstrapScreen(viewModel: BootstrapViewModel) {
    PawAndroidTheme {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.background,
        ) {
            PawNavGraph(viewModel = viewModel)
        }
    }
}
