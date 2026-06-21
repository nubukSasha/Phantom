package com.phantom.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.phantom.model.Direction
import com.phantom.ui.theme.DarkSurfaceVariant
import com.phantom.ui.theme.OnDark
import com.phantom.ui.theme.OnDarkSecondary
import com.phantom.ui.theme.Teal

@Composable
fun ChatBubble(
    text: String,
    direction: Direction,
    timestamp: Long,
    modifier: Modifier = Modifier,
) {
    val isSent = direction == Direction.Sent
    val bubbleColor = if (isSent) Teal.copy(alpha = 0.2f) else DarkSurfaceVariant
    val alignment = if (isSent) Arrangement.End else Arrangement.Start
    val shape = if (isSent) {
        RoundedCornerShape(16.dp, 4.dp, 16.dp, 16.dp)
    } else {
        RoundedCornerShape(4.dp, 16.dp, 16.dp, 16.dp)
    }

    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 2.dp),
        horizontalArrangement = alignment,
    ) {
        Surface(
            shape = shape,
            color = bubbleColor,
            modifier = Modifier.widthIn(max = 280.dp),
        ) {
            Column(modifier = Modifier.padding(12.dp)) {
                Text(
                    text = text,
                    style = MaterialTheme.typography.bodyLarge,
                    color = OnDark,
                )
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                ) {
                    Text(
                        text = formatTime(timestamp),
                        style = MaterialTheme.typography.bodySmall,
                        color = OnDarkSecondary,
                    )
                }
            }
        }
    }
}

private fun formatTime(millis: Long): String {
    val sdf = java.text.SimpleDateFormat("HH:mm", java.util.Locale.getDefault())
    return sdf.format(java.util.Date(millis))
}
