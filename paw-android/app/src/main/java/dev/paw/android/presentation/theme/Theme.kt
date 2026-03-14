package dev.paw.android.presentation.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable

private val PawDarkColors = darkColorScheme(
    primary = PawAccent,
    onPrimary = PawBackground,
    secondary = PawAccent,
    onSecondary = PawBackground,
    background = PawBackground,
    surface = PawSurface1,
    surfaceVariant = PawSurface3,
    onSurface = PawStrongText,
    onSurfaceVariant = PawMutedText,
    outline = PawOutline,
)

private val PawShapes = Shapes(
    extraSmall = RoundedCornerShape(6),
    small = RoundedCornerShape(8),
    medium = RoundedCornerShape(10),
    large = RoundedCornerShape(12),
    extraLarge = RoundedCornerShape(12),
)

@Composable
fun PawAndroidTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = PawDarkColors,
        typography = Typography,
        shapes = PawShapes,
        content = content,
    )
}
