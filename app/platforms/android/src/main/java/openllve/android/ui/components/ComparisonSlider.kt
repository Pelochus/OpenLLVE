package openllve.android.ui.components

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp

/**
 * Original ↔ Enhanced comparison. A slider crossfades from the original
 * (value 0) to the enhanced frame (value 1). Deliberately simple: it is a
 * crossfade, not a custom rendering engine.
 */
@Composable
fun ComparisonSlider(
    original: Bitmap,
    enhanced: Bitmap,
    modifier: Modifier = Modifier,
) {
    var slider by remember { mutableStateOf(0.5f) }

    Column(modifier = modifier.padding(horizontal = 12.dp)) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(16f / 9f),
        ) {
            Image(
                bitmap = original.asImageBitmap(),
                contentDescription = "Original",
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
                alpha = 1f,
            )
            Image(
                bitmap = enhanced.asImageBitmap(),
                contentDescription = "Enhanced",
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
                alpha = slider,
            )
        }

        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(top = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("Original", style = MaterialTheme.typography.labelLarge)
            Spacer(modifier = Modifier.weight(1f))
            Text("Enhanced", style = MaterialTheme.typography.labelLarge)
        }
        Slider(
            value = slider,
            onValueChange = { slider = it },
            modifier = Modifier.fillMaxWidth(),
        )
    }
}
