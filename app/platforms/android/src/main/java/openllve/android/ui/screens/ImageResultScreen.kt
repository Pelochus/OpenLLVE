package openllve.android.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import openllve.android.ui.components.BackendInfoCard
import openllve.android.ui.components.ComparisonSlider
import openllve.android.ui.components.ErrorBanner
import openllve.android.ui.components.MetricsCard
import openllve.android.ui.components.ProcessingIndicator
import openllve.android.ui.components.SettingsCard
import openllve.android.ui.state.UiState
import openllve.android.ui.viewmodel.EnhancementViewModel

/**
 * Image result: original ↔ enhanced comparison, metrics, backend info, and a
 * re-run action. Shows loading / error / empty states as appropriate.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImageResultScreen(
    viewModel: EnhancementViewModel,
    uiState: UiState,
    onBack: () -> Unit
) {
    val original = uiState.imageOriginal
    val enhanced = uiState.imageEnhanced

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Image result") },
                navigationIcon = {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                        contentDescription = "Back",
                        modifier = Modifier.clickable { onBack() }
                    )
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 4.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            when {
                uiState.error != null -> ErrorBanner(uiState.error)
                uiState.processing -> ProcessingIndicator("Enhancing image…")
                original != null && enhanced != null -> {
                    ComparisonSlider(original = original, enhanced = enhanced)
                    uiState.metrics?.let { MetricsCard(it) }
                    uiState.backendSelection?.let { BackendInfoCard(it) }
                    SettingsCard(
                        settings = uiState.settings,
                        probe = uiState.backendProbe,
                        onSettingsChange = viewModel::updateSettings
                    )
                    Button(onClick = viewModel::rerunImage, modifier = Modifier.fillMaxWidth()) {
                        Text("Re-run with current settings")
                    }
                }
                else -> {
                    Text(
                        text = "No image loaded. Go back and select an image.",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}
