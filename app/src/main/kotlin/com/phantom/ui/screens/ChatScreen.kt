package com.phantom.ui.screens

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.phantom.ui.components.ChatBubble
import com.phantom.ui.components.MessageInput
import com.phantom.ui.components.PhantomTopBar
import com.phantom.ui.theme.OnDarkSecondary
import com.phantom.viewmodel.ChatEvent
import com.phantom.viewmodel.ChatViewModel

@Composable
fun ChatScreen(
    contactId: Long,
    onBack: () -> Unit,
    viewModel: ChatViewModel = viewModel(),
) {
    val state by viewModel.state.collectAsState()
    val navigation by viewModel.navigation.collectAsState()
    val listState = rememberLazyListState()
    var showDeleteDialog by remember { mutableStateOf(false) }

    LaunchedEffect(contactId) {
        viewModel.onEvent(ChatEvent.Load(contactId))
    }

    LaunchedEffect(navigation) {
        when (navigation) {
            ChatEvent.NavigateBack -> onBack()
            else -> {}
        }
        viewModel.onNavigated()
    }

    LaunchedEffect(state.messages.size) {
        if (state.messages.isNotEmpty()) {
            listState.animateScrollToItem(0)
        }
    }

    if (showDeleteDialog) {
        AlertDialog(
            onDismissRequest = { showDeleteDialog = false },
            title = { Text("Delete conversation?") },
            confirmButton = {
                TextButton(onClick = {
                    showDeleteDialog = false
                    onBack()
                }) { Text("Delete") }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = false }) { Text("Cancel") }
            },
        )
    }

    Scaffold(
        topBar = {
            PhantomTopBar(
                title = state.contactAlias,
                onBack = onBack,
                actions = {
                    androidx.compose.material3.IconButton(onClick = { showDeleteDialog = true }) {
                        androidx.compose.material3.Icon(
                            Icons.Default.Delete,
                            contentDescription = "Delete",
                            tint = OnDarkSecondary,
                        )
                    }
                },
            )
        },
        bottomBar = {
            MessageInput(
                text = state.inputText,
                onTextChange = { viewModel.onEvent(ChatEvent.UpdateInput(it)) },
                onSend = { viewModel.onEvent(ChatEvent.Send) },
                isSending = state.isSending,
            )
        },
    ) { padding ->
        LazyColumn(
            state = listState,
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
            reverseLayout = true,
        ) {
            items(state.messages, key = { it.messageId }) { msg ->
                ChatBubble(
                    text = msg.text,
                    direction = msg.direction,
                    timestamp = msg.createdAt,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
}
