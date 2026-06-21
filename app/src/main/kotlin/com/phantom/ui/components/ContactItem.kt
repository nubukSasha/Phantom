package com.phantom.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.material3.ExperimentalMaterial3Api
import com.phantom.model.Chat
import com.phantom.ui.theme.OnlineGreen
import com.phantom.ui.theme.OnDark
import com.phantom.ui.theme.OnDarkSecondary

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ContactItem(
    chat: Chat,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = 16.dp, vertical = 12.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Surface(
                modifier = Modifier
                    .size(48.dp)
                    .clip(CircleShape),
                color = if (chat.isOnline) OnlineGreen else MaterialTheme.colorScheme.surfaceVariant,
            ) {
                Text(
                    text = chat.alias.take(1).uppercase(),
                    modifier = Modifier.padding(12.dp),
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
            Spacer(modifier = Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = chat.alias,
                        style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Medium),
                        color = OnDark,
                        modifier = Modifier.weight(1f),
                    )
                    if (chat.lastMessageAt > 0) {
                        Text(
                            text = formatTime(chat.lastMessageAt),
                            style = MaterialTheme.typography.bodySmall,
                            color = OnDarkSecondary,
                        )
                    }
                }
                Spacer(modifier = Modifier.height(4.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = chat.lastMessage ?: "",
                        style = MaterialTheme.typography.bodyMedium,
                        color = OnDarkSecondary,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f),
                    )
                    if (chat.unreadCount > 0) {
                        BadgedBox(badge = {
                            Badge { Text("${chat.unreadCount}") }
                        }) {}
                    }
                }
            }
        }
    }
}

private fun formatTime(millis: Long): String {
    val sdf = java.text.SimpleDateFormat("HH:mm", java.util.Locale.getDefault())
    return sdf.format(java.util.Date(millis))
}
