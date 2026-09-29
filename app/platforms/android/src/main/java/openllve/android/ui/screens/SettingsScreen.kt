package openllve.android.ui.screens

import android.net.Uri
import android.os.Build
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
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
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.example.openllve.BuildConfig
import openllve.android.log.AppLog
import openllve.android.ui.components.ComputeTargetCard
import openllve.android.ui.components.ToppingsCard
import openllve.android.ui.viewmodel.EnhancementViewModel
import openllve.shared.ui.UiState

/**
 * Settings screen: the app's configuration (default compute target +
 * toppings), upcoming features, and app/model/device information. The home
 * screen only shows a read-only summary of the active configuration, so the
 * two screens are distinct.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    viewModel: EnhancementViewModel,
    uiState: UiState,
    onBack: () -> Unit,
    onOpenLogs: () -> Unit,
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
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
            ComputeTargetCard(
                settings = uiState.settings,
                probe = uiState.backendProbe,
                onSettingsChange = viewModel::updateSettings,
            )

            ToppingsCard(
                settings = uiState.settings,
                onSettingsChange = viewModel::updateSettings,
            )

            ComingSoonCard()

            LogsCard(onViewLogs = onOpenLogs)

            AboutCard()

            PrivacyCard()
        }
    }
}

/**
 * Log access: view the in-memory buffer, save a copy to a file (Storage
 * Access Framework), or clear it.
 */
@Composable
private fun LogsCard(onViewLogs: () -> Unit) {
    val context = LocalContext.current
    val saveLogs =
        rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("text/plain")) { uri: Uri? ->
            if (uri != null) {
                try {
                    val text = AppLog.snapshot().joinToString("\n")
                    context.contentResolver.openOutputStream(uri)?.use {
                        it.write(text.toByteArray(Charsets.UTF_8))
                    }
                    Toast.makeText(context, "Logs saved", Toast.LENGTH_SHORT).show()
                } catch (e: Exception) {
                    Toast.makeText(context, "Could not save logs: ${e.message ?: "unknown error"}", Toast.LENGTH_SHORT).show()
                }
            }
        }

    Card(modifier = Modifier.padding(horizontal = 12.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Logs", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text =
                    "In-memory buffer of the last ${AppLog.MAX_ENTRIES} app events " +
                        "(mirrored to logcat as tag ${AppLog.LOGCAT_TAG}). " +
                        "Cleared on app restart; use “Save to file” to keep a copy.",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.height(8.dp))
            Row {
                Button(onClick = onViewLogs) {
                    Text("View logs")
                }
                Spacer(modifier = Modifier.width(8.dp))
                Button(onClick = { saveLogs.launch("openllve-logs.txt") }) {
                    Text("Save to file…")
                }
            }
            Spacer(modifier = Modifier.height(8.dp))
            Button(onClick = { AppLog.clear() }) {
                Text("Clear")
            }
        }
    }
}

/** Placeholder toggles for features planned in later phases. */
@Composable
private fun ComingSoonCard() {
    Card(modifier = Modifier.padding(horizontal = 12.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Coming soon", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))
            ComingSoonRow("Adaptive exposure compensation")
            ComingSoonRow("Temporal noise suppression")
            ComingSoonRow("Automatic backend selection")
        }
    }
}

@Composable
private fun ComingSoonRow(label: String) {
    Row(modifier = Modifier.padding(vertical = 4.dp)) {
        Column(modifier = Modifier.weight(1f)) {
            Text(text = label, style = MaterialTheme.typography.bodyMedium)
            Text(
                text = "planned for a later phase",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Switch(checked = false, enabled = false, onCheckedChange = {})
    }
}

/** App, model, runtime, and device information. */
@Composable
private fun AboutCard() {
    Card(modifier = Modifier.padding(horizontal = 12.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("About", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))
            AboutRow("Version", BuildConfig.VERSION_NAME)
            AboutRow("Model", "Zero-DCE (DCE-Net), INT8")
            Text(
                text =
                    "256×256 patch tiling with 16-pixel overlap; 24-channel " +
                        "learned-curve output per patch.",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.height(8.dp))
            AboutRow("Runtime", "LiteRT 2.2.0 (Google AI Edge LiteRT)")
            AboutRow("Rust core", "pending (TODO.md P1.1)")
            Spacer(modifier = Modifier.height(8.dp))
            AboutRow("Device", "${Build.MANUFACTURER} ${Build.MODEL}")
            AboutRow("Android", "API ${Build.VERSION.SDK_INT} (${Build.VERSION.RELEASE})")
            // Build.SOC_MODEL only exists from API 35; minSdk is 26.
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.VANILLA_ICE_CREAM) {
                AboutRow("SoC", Build.SOC_MODEL)
            }
        }
    }
}

@Composable
private fun AboutRow(label: String, value: String) {
    Row(modifier = Modifier.padding(vertical = 2.dp)) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.weight(1f),
        )
        Text(text = value, style = MaterialTheme.typography.labelMedium)
    }
}

/** On-device processing and local storage note. */
@Composable
private fun PrivacyCard() {
    Card(modifier = Modifier.padding(horizontal = 12.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Privacy", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text =
                    "All processing happens on-device. Your images and videos " +
                        "never leave the phone; settings are stored locally with " +
                        "Jetpack DataStore.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
