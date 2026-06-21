package com.phantom.ui.screens

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.google.zxing.BarcodeFormat
import com.google.zxing.qrcode.QRCodeWriter
import com.phantom.PhantomApp
import com.phantom.ui.components.PhantomTopBar
import com.phantom.ui.theme.OnDark
import com.phantom.ui.theme.OnDarkSecondary

@Composable
fun QrDisplayScreen(onBack: () -> Unit) {
    val core = PhantomApp.instance.core
    val qrContent = remember {
        "PHANTOM:${core.identityPublic().joinToString("") { "%02x".format(it) }}:" +
            "${core.onionAddress()}"
    }
    var qrBitmap by remember { mutableStateOf<Bitmap?>(null) }

    LaunchedEffect(qrContent) {
        try {
            val writer = QRCodeWriter()
            val bitMatrix = writer.encode(qrContent, BarcodeFormat.QR_CODE, 512, 512)
            val bitmap = Bitmap.createBitmap(512, 512, Bitmap.Config.RGB_565)
            for (x in 0 until 512) {
                for (y in 0 until 512) {
                    bitmap.setPixel(x, y, if (bitMatrix[x, y]) -0x1000000 else -0x1)
                }
            }
            qrBitmap = bitmap
        } catch (_: Exception) { }
    }

    Scaffold(
        topBar = { PhantomTopBar(title = "My QR", onBack = onBack) },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Spacer(modifier = Modifier.height(48.dp))

            qrBitmap?.let {
                Image(
                    bitmap = it.asImageBitmap(),
                    contentDescription = "QR Code",
                    modifier = Modifier.size(256.dp),
                )
            }

            Spacer(modifier = Modifier.height(24.dp))

            Text(
                text = core.onionAddress(),
                style = androidx.compose.material3.MaterialTheme.typography.labelMedium,
                fontFamily = FontFamily.Monospace,
                color = OnDark,
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(horizontal = 32.dp),
            )
        }
    }
}
