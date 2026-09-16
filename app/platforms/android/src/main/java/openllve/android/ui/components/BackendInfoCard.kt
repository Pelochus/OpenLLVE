package openllve.android.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import openllve.android.domain.BackendSelection

/**
 * Reports which backend was actually used vs. requested. Never silent: when
 * the engine fell back, the reason is shown (e.g. "NPU delegate unavailable").
 */
@Composable
fun BackendInfoCard(selection: BackendSelection, modifier: Modifier = Modifier) {
    Card(modifier = modifier.padding(horizontal = 12.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Compute backend", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = selection.summary,
                style = MaterialTheme.typography.bodyMedium
            )
            if (selection.fellBack) {
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = selection.reason ?: "Fell back to a supported backend",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.error
                )
            }
        }
    }
}
