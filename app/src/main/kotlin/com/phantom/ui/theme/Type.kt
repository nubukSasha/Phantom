package com.phantom.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

val PhantomTypography = Typography(
    titleLarge = TextStyle(
        fontWeight = FontWeight.SemiBold,
        fontSize = 18.sp,
    ),
    titleMedium = TextStyle(
        fontWeight = FontWeight.Medium,
        fontSize = 16.sp,
    ),
    bodyLarge = TextStyle(
        fontSize = 16.sp,
    ),
    bodyMedium = TextStyle(
        fontSize = 14.sp,
    ),
    bodySmall = TextStyle(
        fontSize = 12.sp,
    ),
    labelMedium = TextStyle(
        fontFamily = FontFamily.Monospace,
        fontSize = 12.sp,
    ),
)
