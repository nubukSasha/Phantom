package com.phantom.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Divider
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.phantom.ui.components.PhantomTopBar
import com.phantom.ui.theme.ErrorRed
import com.phantom.viewmodel.SettingsEvent
import com.phantom.viewmodel.SettingsViewModel

@Composable
fun SettingsScreen(
    onBack: () -> Unit,
    onShowQr: () -> Unit,
    viewModel: SettingsViewModel = viewModel(),
) {
    val state by viewModel.state.collectAsState()
    val navigation by viewModel.navigation.collectAsState()

    LaunchedEffect(navigation) {
        when (navigation) {
            SettingsEvent.NavigateBack -> onBack()
            else -> {}
        }
        viewModel.onNavigated()
    }

    if (state.showDestroyConfirm) {
        AlertDialog(
            onDismissRequest = { viewModel.onEvent(SettingsEvent.DismissDestroyConfirm) },
            title = { Text("Wipe all data?") },
            text = { Text("This cannot be undone. All contacts and messages will be permanently deleted.") },
            confirmButton = {
                TextButton(onClick = { viewModel.onEvent(SettingsEvent.ConfirmDestroy) }) {
                    Text("Wipe", color = ErrorRed)
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.onEvent(SettingsEvent.DismissDestroyConfirm) }) {
                    Text("Cancel")
                }
            },
        )
    }

    Scaffold(
        topBar = { PhantomTopBar(title = "Settings", onBack = onBack) },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
        ) {
            SectionHeader("Identity")
            InfoRow(
                label = "Public Key",
                value = state.identityKeyHex,
                onCopy = { viewModel.onEvent(SettingsEvent.CopyIdentityKey) },
            )
            InfoRow(
                label = "Onion Address",
                value = state.onionAddress,
                onCopy = { viewModel.onEvent(SettingsEvent.CopyOnionAddress) },
            )
            Button(
                onClick = onShowQr,
                modifier = Modifier.fillMaxWidth(),
            ) { Text("Show QR Code") }

            Spacer(modifier = Modifier.height(24.dp))
            Divider()
            Spacer(modifier = Modifier.height(24.dp))

            SectionHeader("Security")
            Button(
                onClick = { viewModel.onEvent(SettingsEvent.ToggleLock) },
                modifier = Modifier.fillMaxWidth(),
            ) { Text(if (state.isLocked) "Unlock Storage" else "Lock Storage") }

            Spacer(modifier = Modifier.height(24.dp))
            Divider()
            Spacer(modifier = Modifier.height(24.dp))

            SectionHeader("Danger Zone")
            Button(
                onClick = { viewModel.onEvent(SettingsEvent.ShowDestroyConfirm) },
                colors = ButtonDefaults.buttonColors(containerColor = ErrorRed),
                modifier = Modifier.fillMaxWidth(),
            ) { Text("Wipe All Data") }
        }
    }
}

@Composable
private fun SectionHeader(text: String) {
    Text(
        text = text,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(bottom = 8.dp),
    )
}

@Composable
private fun InfoRow(
    label: String,
    value: String,
    onCopy: () -> Unit,
) {
    Column(modifier = Modifier.padding(vertical = 8.dp)) {
        Text(text = label, color = MaterialTheme.colorScheme.onSurfaceVariant, style = androidx.compose.material3.MaterialTheme.typography.bodySmall)
        Text(
            text = value,
            color = MaterialTheme.colorScheme.onSurface,
            fontFamily = FontFamily.Monospace,
            style = androidx.compose.material3.MaterialTheme.typography.bodyMedium,
            modifier = Modifier.padding(vertical = 4.dp),
        )
        TextButton(onClick = onCopy) { Text("Copy") }
    }
}
