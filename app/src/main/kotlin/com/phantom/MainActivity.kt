package com.phantom

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.core.content.ContextCompat
import androidx.lifecycle.lifecycleScope
import com.phantom.service.PhantomForegroundService
import com.phantom.ui.navigation.PhantomNavGraph
import com.phantom.ui.theme.PhantomTheme
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {

    private val requestPermission = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { /* granted or denied */ }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        requestRequiredPermissions()
        startForegroundService()

        setContent {
            PhantomTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background,
                ) {
                    PhantomNavGraph()
                }
            }

            LaunchedEffect(Unit) {
                PhantomApp.instance.initCore()
            }
        }
    }

    private fun requestRequiredPermissions() {
        val perms = mutableListOf<String>()
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS)
            != PackageManager.PERMISSION_GRANTED && Build.VERSION.SDK_INT >= 33
        ) {
            perms.add(Manifest.permission.POST_NOTIFICATIONS)
        }
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA)
            != PackageManager.PERMISSION_GRANTED
        ) {
            perms.add(Manifest.permission.CAMERA)
        }
        if (perms.isNotEmpty()) {
            requestPermission.launch(perms.toTypedArray())
        }
    }

    private fun startForegroundService() {
        val intent = android.content.Intent(
            this, PhantomForegroundService::class.java
        )
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
    }
}
