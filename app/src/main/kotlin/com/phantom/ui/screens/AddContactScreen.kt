package com.phantom.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import com.phantom.PhantomApp
import com.phantom.ui.components.PhantomTopBar
import com.phantom.ui.theme.ErrorRed
import kotlinx.coroutines.launch

@Composable
fun AddContactScreen(
    onBack: () -> Unit,
    onScan: () -> Unit,
) {
    var alias by remember { mutableStateOf("") }
    var identityPk by remember { mutableStateOf("") }
    var staticPk by remember { mutableStateOf("") }
    var onionAddress by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var isSaving by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    fun parseHex(hex: String): ByteArray? {
        return try {
            hex.replace(" ", "").chunked(2).map { it.toInt(16).toByte() }.toByteArray()
        } catch (_: Exception) { null }
    }

    Scaffold(
        topBar = { PhantomTopBar(title = "Add Contact", onBack = onBack) },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
        ) {
            OutlinedTextField(
                value = alias,
                onValueChange = { alias = it },
                label = { Text("Alias") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(modifier = Modifier.height(12.dp))
            OutlinedTextField(
                value = identityPk,
                onValueChange = { identityPk = it },
                label = { Text("Identity Public Key (hex)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                textStyle = androidx.compose.ui.text.TextStyle(fontFamily = FontFamily.Monospace),
            )
            Spacer(modifier = Modifier.height(12.dp))
            OutlinedTextField(
                value = staticPk,
                onValueChange = { staticPk = it },
                label = { Text("Static Public Key (hex)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                textStyle = androidx.compose.ui.text.TextStyle(fontFamily = FontFamily.Monospace),
            )
            Spacer(modifier = Modifier.height(12.dp))
            OutlinedTextField(
                value = onionAddress,
                onValueChange = { onionAddress = it },
                label = { Text("Onion Address") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                textStyle = androidx.compose.ui.text.TextStyle(fontFamily = FontFamily.Monospace),
            )
            Spacer(modifier = Modifier.height(12.dp))

            if (error != null) {
                Text(text = error!!, color = ErrorRed)
                Spacer(modifier = Modifier.height(8.dp))
            }

            Button(
                onClick = onScan,
                modifier = Modifier.fillMaxWidth(),
            ) { Text("Scan QR Code") }
            Spacer(modifier = Modifier.height(8.dp))
            Button(
                onClick = {
                    scope.launch {
                        isSaving = true
                        error = null
                        try {
                            val idBytes = parseHex(identityPk)
                            val stBytes = parseHex(staticPk)
                            if (idBytes?.size != 32) { error = "Identity PK: 64 hex chars"; return@launch }
                            if (stBytes?.size != 32) { error = "Static PK: 64 hex chars"; return@launch }
                            PhantomApp.instance.core.addContact(
                                alias = alias,
                                identityPk = idBytes,
                                staticPk = stBytes,
                                onionAddress = onionAddress,
                            )
                            onBack()
                        } catch (e: Exception) {
                            error = e.message
                        } finally {
                            isSaving = false
                        }
                    }
                },
                enabled = alias.isNotBlank() && !isSaving,
                modifier = Modifier.fillMaxWidth(),
            ) { Text(if (isSaving) "Saving…" else "Save") }
        }
    }
}
