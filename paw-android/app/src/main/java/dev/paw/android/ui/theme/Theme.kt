package dev.paw.android.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable

private val PawDarkColors = darkColorScheme(
    primary = PawPrimary,
    onPrimary = Color(0xFF06211A),
    secondary = PawAccent,
    onSecondary = Color(0xFF08161F),
    background = PawBackground,
    surface = PawSurface1,
    surfaceVariant = PawSurface3,
    onSurface = PawStrongText,
    onSurfaceVariant = PawMutedText,
    outline = PawOutline,
)

@Composable
fun PawAndroidTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = PawDarkColors,
        typography = Typography,
        content = content,
    )
}
