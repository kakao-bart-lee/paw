@file:Suppress("unused")
package dev.paw.android

import androidx.compose.runtime.Composable
import dev.paw.android.presentation.bootstrap.BootstrapScreen
import dev.paw.android.presentation.bootstrap.BootstrapViewModel

/**
 * Backward-compatibility entry point.
 * New code should use BootstrapScreen from presentation.bootstrap directly.
 */
@Composable
fun PawAndroidApp(viewModel: BootstrapViewModel) {
    BootstrapScreen(viewModel = viewModel)
}
