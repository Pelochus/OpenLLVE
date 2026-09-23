package openllve.android.ui.screens

import android.net.Uri
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
import androidx.compose.material.icons.filled.Settings
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
import openllve.android.ui.components.SettingsCard
import openllve.android.ui.viewmodel.EnhancementViewModel
import openllve.shared.ui.UiState

/**
 * Entry screen: select an image or MP4, and configure the enhancement
 * (compute target + toppings). Selecting media runs the enhancement and
 * navigates to the result screen.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    viewModel: EnhancementViewModel,
    uiState: UiState,
    onOpenImage: (Uri, String) -> Unit,
    onOpenVideo: (Uri, String) -> Unit,
    onOpenSettings: () -> Unit,
) {
    val pickImage =
        rememberLauncherForActivityResult(
            ActivityResultContracts.OpenDocument(),
        ) { uri: Uri? ->
            if (uri != null) {
                onOpenImage(uri, uri.lastPathSegment ?: "image")
            }
        }
    val pickVideo =
        rememberLauncherForActivityResult(
            ActivityResultContracts.OpenDocument(),
        ) { uri: Uri? ->
            if (uri != null) {
                onOpenVideo(uri, uri.lastPathSegment ?: "video")
            }
        }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("OpenLLVE") },
                actions = {
                    Icon(
                        imageVector = Icons.Filled.Settings,
                        contentDescription = "Settings",
                        modifier = Modifier.clickable { onOpenSettings() },
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
            Text(
                text = "Low-light video enhancement",
                style = MaterialTheme.typography.titleLarge,
            )
            Text(
                text = "Select an image or an MP4 to run the Zero-DCE enhancement model on-device.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            Row(modifier = Modifier.fillMaxWidth()) {
                Button(
                    onClick = { pickImage.launch(arrayOf("image/*")) },
                    modifier = Modifier.weight(1f),
                ) {
                    Text("Select image")
                }
                Spacer(modifier = Modifier.width(12.dp))
                Button(
                    onClick = { pickVideo.launch(arrayOf("video/*")) },
                    modifier = Modifier.weight(1f),
                ) {
                    Text("Select MP4 / video")
                }
            }

            SettingsCard(
                settings = uiState.settings,
                probe = uiState.backendProbe,
                onSettingsChange = viewModel::updateSettings,
            )

            Spacer(modifier = Modifier.height(24.dp))
        }
    }
}
