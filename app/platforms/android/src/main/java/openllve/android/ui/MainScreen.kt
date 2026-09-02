package openllve.android.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun MainScreen() {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("OpenLLVE") },
                actions = {
                    // Add actions here if needed
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Text("Welcome to OpenLLVE")
            Spacer(modifier = Modifier.height(16.dp))
            Button(onClick = { /* TODO: Start video enhancement */ }) {
                Text("Start Enhancement")
            }
        }
    }
}