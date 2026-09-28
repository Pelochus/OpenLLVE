package openllve.android.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import openllve.shared.domain.BackendProbeResult
import openllve.shared.domain.ComputeTarget
import openllve.shared.domain.EnhancementSettings

/**
 * Combined configuration cards (default target + toppings). Used on the
 * result screens, where the user re-runs with the current settings; the
 * settings screen renders the two cards as separate sections.
 */
@Composable
fun SettingsCard(
    settings: EnhancementSettings,
    probe: BackendProbeResult?,
    onSettingsChange: (EnhancementSettings) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        ComputeTargetCard(settings, probe, onSettingsChange)
        ToppingsCard(settings, onSettingsChange)
    }
}

/**
 * Default compute target selector (with unsupported targets disabled and
 * annotated). Lives on the settings screen; the home screen shows a
 * read-only summary of the active configuration instead.
 */
@Composable
fun ComputeTargetCard(
    settings: EnhancementSettings,
    probe: BackendProbeResult?,
    onSettingsChange: (EnhancementSettings) -> Unit,
    modifier: Modifier = Modifier,
) {
    Card(modifier = modifier.padding(horizontal = 12.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Default compute target", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = "Applied to every new run. Unsupported targets are disabled with a reason.",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.height(8.dp))
            ComputeTargetSelector(
                selected = settings.computeTarget,
                probe = probe,
                onSelect = { target -> onSettingsChange(settings.copy(computeTarget = target)) },
            )
        }
    }
}

/**
 * Enhancement toppings: EWMA and flicker reduction toggles. Exposed and
 * persisted here; in this prototype the actual temporal processing is
 * deferred to the Rust core (see TODO.md P1.1), so they are labelled as
 * "applied by the native pipeline".
 */
@Composable
fun ToppingsCard(
    settings: EnhancementSettings,
    onSettingsChange: (EnhancementSettings) -> Unit,
    modifier: Modifier = Modifier,
) {
    Card(modifier = modifier.padding(horizontal = 12.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Enhancement toppings", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))
            ToggleRow(
                label = "EWMA (temporal smoothing)",
                pending = true,
                checked = settings.ewmaEnabled,
                onCheckedChange = { onSettingsChange(settings.copy(ewmaEnabled = it)) },
            )
            ToggleRow(
                label = "Flicker reduction",
                pending = true,
                checked = settings.flickerReductionEnabled,
                onCheckedChange = { onSettingsChange(settings.copy(flickerReductionEnabled = it)) },
            )
        }
    }
}

@Composable
private fun ComputeTargetSelector(
    selected: ComputeTarget,
    probe: BackendProbeResult?,
    onSelect: (ComputeTarget) -> Unit,
) {
    ComputeTarget.entries.forEach { target ->
        val supported = probe?.isSupported(target) ?: true
        val note = probe?.noteFor(target)
        Row(
            modifier = Modifier.padding(vertical = 2.dp),
        ) {
            RadioButton(
                selected = selected == target,
                enabled = supported,
                onClick = { if (supported) onSelect(target) },
            )
            Spacer(modifier = Modifier.width(8.dp))
            Column {
                Text(
                    text = target.label,
                    style = MaterialTheme.typography.bodyMedium,
                    color =
                        if (supported) {
                            MaterialTheme.colorScheme.onSurface
                        } else {
                            MaterialTheme.colorScheme.onSurface.copy(alpha = 0.5f)
                        },
                )
                if (target == ComputeTarget.XNNPACK) {
                    Text(
                        text = "no XNNPACK accelerator in LiteRT; runs on CPU",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (!supported && note != null) {
                    Text(
                        text = note,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }
        }
    }
}

@Composable
private fun ToggleRow(
    label: String,
    pending: Boolean,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(modifier = Modifier.padding(vertical = 4.dp)) {
        Column(modifier = Modifier.weight(1f)) {
            Text(text = label, style = MaterialTheme.typography.bodyMedium)
            if (pending) {
                Text(
                    text = "prototype — applied by the native pipeline",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        Switch(checked = checked, onCheckedChange = onCheckedChange)
    }
}
