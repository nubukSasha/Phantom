package com.phantom.ui.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import com.phantom.ui.theme.DarkSurface
import com.phantom.ui.theme.OnDark

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PhantomTopBar(
    title: String,
    onBack: (() -> Unit)? = null,
    actions: @Composable () -> Unit = {},
) {
    CenterAlignedTopAppBar(
        title = { Text(title) },
        navigationIcon = {
            if (onBack != null) {
                IconButton(onClick = onBack) {
                    Icon(
                        imageVector = Icons.Default.ArrowBack,
                        contentDescription = "Back",
                        tint = OnDark,
                    )
                }
            }
        },
        actions = { actions() },
        colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
            containerColor = DarkSurface,
            titleContentColor = OnDark,
        ),
    )
}
