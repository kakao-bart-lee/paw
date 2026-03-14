@file:Suppress("unused")
package dev.paw.android.ui.theme

import androidx.compose.runtime.Composable

/**
 * Backward-compatibility re-export. New code should import from presentation.theme directly.
 */
@Composable
fun PawAndroidTheme(content: @Composable () -> Unit) {
    dev.paw.android.presentation.theme.PawAndroidTheme(content)
}
