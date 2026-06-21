package com.phantom.ui.components

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.phantom.ui.theme.DarkSurfaceVariant
import com.phantom.ui.theme.OnDark
import com.phantom.ui.theme.OnDarkSecondary
import com.phantom.ui.theme.Teal

@Composable
fun MessageInput(
    text: String,
    onTextChange: (String) -> Unit,
    onSend: () -> Unit,
    isSending: Boolean,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        OutlinedTextField(
            value = text,
            onValueChange = onTextChange,
            placeholder = { Text("Type a message…", color = OnDarkSecondary) },
            modifier = Modifier.weight(1f),
            shape = RoundedCornerShape(24.dp),
            colors = OutlinedTextFieldDefaults.colors(
                focusedBorderColor = Teal,
                unfocusedBorderColor = DarkSurfaceVariant,
                focusedTextColor = OnDark,
                unfocusedTextColor = OnDark,
                cursorColor = Teal,
            ),
            singleLine = true,
            enabled = !isSending,
        )
        IconButton(
            onClick = onSend,
            enabled = text.isNotBlank() && !isSending,
        ) {
            Icon(
                imageVector = Icons.Default.Send,
                contentDescription = "Send",
                tint = if (text.isNotBlank() && !isSending) Teal else OnDarkSecondary,
            )
        }
    }
}
