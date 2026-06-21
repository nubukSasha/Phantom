package com.phantom.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable

private val DarkColors = darkColorScheme(
    primary = Teal,
    onPrimary = DarkBackground,
    primaryContainer = TealDark,
    secondary = Teal,
    background = DarkBackground,
    surface = DarkSurface,
    surfaceVariant = DarkSurfaceVariant,
    onBackground = OnDark,
    onSurface = OnDark,
    onSurfaceVariant = OnDarkSecondary,
    error = ErrorRed,
    onError = DarkBackground,
)

@Composable
fun PhantomTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = DarkColors,
        typography = PhantomTypography,
        content = content,
    )
}
