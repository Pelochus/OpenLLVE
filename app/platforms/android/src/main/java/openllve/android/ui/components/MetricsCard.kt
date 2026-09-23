package openllve.android.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import openllve.shared.domain.ProcessingMetrics

/**
 * Minimal, useful performance metrics: total inference time, frame count,
 * FPS, average frame time, backend, model, input resolution, and (for video)
 * whether processing is faster/slower than realtime.
 */
@Composable
fun MetricsCard(
    metrics: ProcessingMetrics,
    modifier: Modifier = Modifier,
) {
    Card(modifier = modifier.padding(horizontal = 12.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Performance", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))
            MetricRow("Frames processed", metrics.frameCount.toString())
            MetricRow("Total inference", "%.1f ms".format(metrics.inferenceMs))
            MetricRow("Avg per frame", "%.1f ms".format(metrics.averageFrameMs))
            MetricRow("Throughput", "%.1f FPS".format(metrics.fps))
            MetricRow("Input resolution", metrics.inputResolution)
            MetricRow("Backend", metrics.backend.label)
            MetricRow("Model", metrics.modelName)
            metrics.realtimeLabel?.let {
                Spacer(modifier = Modifier.height(4.dp))
                MetricRow("vs realtime", it)
            }
        }
    }
}

@Composable
private fun MetricRow(
    label: String,
    value: String,
) {
    Row {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.width(160.dp),
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}
