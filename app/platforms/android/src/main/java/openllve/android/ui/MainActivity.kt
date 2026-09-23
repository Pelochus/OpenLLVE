package openllve.android.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import openllve.android.OpenLLVEApp
import openllve.android.ui.navigation.Destination
import openllve.android.ui.screens.HomeScreen
import openllve.android.ui.screens.ImageResultScreen
import openllve.android.ui.screens.SettingsScreen
import openllve.android.ui.screens.VideoResultScreen
import openllve.android.ui.theme.OpenLLVETheme
import openllve.android.ui.viewmodel.EnhancementViewModel

/**
 * The single launcher activity. It hosts the Compose UI and wires the
 * application-scoped components (engine + settings) into the ViewModel.
 */
class MainActivity : ComponentActivity() {
    private val app get() = application as OpenLLVEApp

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            OpenLLVETheme {
                val navController = rememberNavController()
                val viewModel: EnhancementViewModel =
                    viewModel(
                        factory =
                            EnhancementViewModel.factory(
                                context = this,
                                engine = app.enhancementEngine,
                                settingsRepository = app.settingsRepository,
                            ),
                    )
                val uiState by viewModel.uiState.collectAsStateWithLifecycle()
                NavHost(
                    navController = navController,
                    startDestination = Destination.HOME,
                ) {
                    composable(Destination.HOME) {
                        HomeScreen(
                            viewModel = viewModel,
                            uiState = uiState,
                            onOpenImage = { uri, name ->
                                viewModel.selectImage(uri, name)
                                navController.navigate(Destination.IMAGE_RESULT)
                            },
                            onOpenVideo = { uri, name ->
                                viewModel.selectVideo(uri, name)
                                navController.navigate(Destination.VIDEO_RESULT)
                            },
                            onOpenSettings = { navController.navigate(Destination.SETTINGS) },
                        )
                    }
                    composable(Destination.SETTINGS) {
                        SettingsScreen(
                            viewModel = viewModel,
                            uiState = uiState,
                            onBack = { navController.popBackStack() },
                        )
                    }
                    composable(Destination.IMAGE_RESULT) {
                        ImageResultScreen(
                            viewModel = viewModel,
                            uiState = uiState,
                            onBack = { navController.popBackStack() },
                        )
                    }
                    composable(Destination.VIDEO_RESULT) {
                        VideoResultScreen(
                            viewModel = viewModel,
                            uiState = uiState,
                            onBack = { navController.popBackStack() },
                        )
                    }
                }
            }
        }
    }
}
