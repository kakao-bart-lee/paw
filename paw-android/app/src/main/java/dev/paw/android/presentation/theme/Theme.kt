package dev.paw.android.presentation.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp

private val PawDarkColors = darkColorScheme(
    primary = PawPrimary,
    onPrimary = PawPrimaryForeground,
    secondary = PawAccent,
    onSecondary = PawPrimaryForeground,
    background = PawBackground,
    onBackground = PawStrongText,
    surface = PawSurface1,
    surfaceVariant = PawSurface3,
    onSurface = PawStrongText,
    onSurfaceVariant = PawMutedText,
    outline = PawOutline,
    error = PawDestructive,
    onError = PawStrongText,
)

private val PawShapes = Shapes(
    extraSmall = RoundedCornerShape(6.dp),
    small = RoundedCornerShape(8.dp),
    medium = RoundedCornerShape(12.dp),
    large = RoundedCornerShape(16.dp),
    extraLarge = RoundedCornerShape(24.dp),
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
