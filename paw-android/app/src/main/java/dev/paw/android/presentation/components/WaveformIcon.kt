package dev.paw.android.presentation.components

import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import kotlin.math.sin

@Composable
fun WaveformIcon(
    signature: String,
    color: Color,
    modifier: Modifier = Modifier,
) {
    Canvas(modifier = modifier) {
        val w = size.width
        val h = size.height
        val cy = h / 2f
        val stroke = Stroke(width = 1.8.dp.toPx(), cap = StrokeCap.Round)

        when (signature) {
            "sine" -> {
                val path = Path()
                path.moveTo(0f, cy)
                val steps = 60
                for (i in 0..steps) {
                    val x = w * i / steps
                    val y = cy + sin(i * 0.18f) * h * 0.35f
                    path.lineTo(x, y)
                }
                drawPath(path, color, style = stroke)
            }
            "pulse" -> {
                val path = Path()
                path.moveTo(0f, cy)
                path.lineTo(w * 0.2f, cy)
                path.lineTo(w * 0.28f, h * 0.15f)
                path.lineTo(w * 0.36f, h * 0.85f)
                path.lineTo(w * 0.42f, h * 0.25f)
                path.lineTo(w * 0.48f, h * 0.7f)
                path.lineTo(w * 0.55f, cy)
                path.lineTo(w * 0.65f, cy)
                path.lineTo(w * 0.72f, h * 0.3f)
                path.lineTo(w * 0.78f, h * 0.75f)
                path.lineTo(w * 0.84f, cy)
                path.lineTo(w, cy)
                drawPath(path, color, style = stroke)
            }
            "wave" -> {
                val path = Path()
                path.moveTo(0f, cy)
                val steps = 60
                for (i in 0..steps) {
                    val x = w * i / steps
                    val amp = h * 0.25f * (1f - (i.toFloat() / steps - 0.5f).let { it * it } * 2f)
                    val y = cy + sin(i * 0.25f) * amp
                    path.lineTo(x, y)
                }
                drawPath(path, color, style = stroke)
            }
            "fractal" -> {
                val path = Path()
                path.moveTo(0f, cy)
                val segments = 12
                for (i in 0..segments) {
                    val x = w * i / segments
                    val offset = if (i % 2 == 0) -h * 0.3f else h * 0.3f
                    path.lineTo(x, cy + offset)
                }
                drawPath(path, color, style = stroke)
            }
            else -> {
                drawLine(color, Offset(0f, cy), Offset(w, cy), strokeWidth = stroke.width)
            }
        }
    }
}
