package openllve.android.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import openllve.android.media.toBitmap
import openllve.android.ui.components.BackendInfoCard
import openllve.android.ui.components.ComparisonSlider
import openllve.android.ui.components.ErrorBanner
import openllve.android.ui.components.MetricsCard
import openllve.android.ui.components.ProcessingIndicator
import openllve.android.ui.components.SettingsCard
import openllve.android.ui.viewmodel.EnhancementViewModel
import openllve.shared.domain.ProcessingMetrics
import openllve.shared.media.VideoMetadata
import openllve.shared.ui.UiState

/**
 * Video result: metadata, configuration, and (once started) synchronized
 * original ↔ enhanced playback with live metrics. Synchronized playback with
 * audio is deferred to the Rust/native pipeline (TODO.md P1.1).
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VideoResultScreen(
    viewModel: EnhancementViewModel,
    uiState: UiState,
    onBack: () -> Unit,
) {
    val original = uiState.videoOriginal?.toBitmap()
    val enhanced = uiState.videoEnhanced?.toBitmap()
    val metrics = videoMetrics(uiState)

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Video result") },
                navigationIcon = {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                        contentDescription = "Back",
                        modifier = Modifier.clickable { onBack() },
                    )
                },
            )
        },
    ) { innerPadding ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(innerPadding)
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 4.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            uiState.error?.let { ErrorBanner(it) }

            uiState.videoMetadata?.let { MetadataCard(it) }

            when {
                uiState.videoPlaying -> {
                    if (original != null && enhanced != null) {
                        ComparisonSlider(original = original, enhanced = enhanced)
                    } else {
                        ProcessingIndicator("Decoding and enhancing…")
                    }
                    metrics?.let { MetricsCard(it) }
                    uiState.videoBackend?.let { BackendInfoCard(it) }
                    Button(onClick = viewModel::stopVideo, modifier = Modifier.fillMaxWidth()) {
                        Text("Stop")
                    }
                }

                else -> {
                    SettingsCard(
                        settings = uiState.settings,
                        probe = uiState.backendProbe,
                        onSettingsChange = viewModel::updateSettings,
                    )
                    Button(onClick = viewModel::startVideo, modifier = Modifier.fillMaxWidth()) {
                        Text("Start enhancement")
                    }
                }
            }
        }
    }
}

@Composable
private fun MetadataCard(metadata: VideoMetadata) {
    androidx.compose.material3.Card(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Video", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))
            Text("Resolution: ${metadata.resolution}")
            Text("Duration: ${metadata.durationLabel}")
            Text("Format: ${metadata.mimeType}")
            Text("Size: ${(metadata.fileSizeBytes / 1024.0 / 1024.0).let { "%.1f MB".format(it) }}")
        }
    }
}

private fun videoMetrics(uiState: UiState): ProcessingMetrics? {
    val count = uiState.videoFrameCount
    if (count == 0) return null
    val inferenceMs = uiState.videoInferenceMs
    val durationMs = uiState.videoMetadata?.durationMs ?: 0L
    return ProcessingMetrics(
        frameCount = count,
        inferenceMs = inferenceMs,
        averageFrameMs = if (count > 0) inferenceMs.toDouble() / count else 0.0,
        fps = if (inferenceMs > 0) count.toDouble() * 1000.0 / inferenceMs else 0.0,
        inputResolution = uiState.videoMetadata?.resolution ?: "unknown",
        backend = uiState.videoBackend?.actual ?: uiState.settings.computeTarget,
        modelName = "zero-dce-int8",
        realtimeFactor =
            if (inferenceMs > 0 && durationMs > 0) {
                durationMs.toDouble() / inferenceMs
            } else {
                0.0
            },
    )
}
