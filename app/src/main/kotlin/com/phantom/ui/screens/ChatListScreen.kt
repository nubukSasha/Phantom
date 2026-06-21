package com.phantom.ui.screens

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.phantom.ui.components.ContactItem
import com.phantom.ui.components.PhantomTopBar
import com.phantom.ui.theme.OnDarkSecondary
import com.phantom.viewmodel.ChatListEvent
import com.phantom.viewmodel.ChatListViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatListScreen(
    onOpenChat: (Long) -> Unit,
    onAddContact: () -> Unit,
    onSettings: () -> Unit,
    viewModel: ChatListViewModel = viewModel(),
) {
    val state by viewModel.state.collectAsState()
    val navigation by viewModel.navigation.collectAsState()

    LaunchedEffect(navigation) {
        when (navigation) {
            is ChatListEvent.OpenChat -> onOpenChat((navigation as ChatListEvent.OpenChat).contactId)
            ChatListEvent.NavigateToAddContact -> onAddContact()
            ChatListEvent.NavigateToSettings -> onSettings()
            else -> {}
        }
        viewModel.onNavigated()
    }

    Scaffold(
        topBar = {
            PhantomTopBar(
                title = "Phantom",
                actions = {
                    IconButton(onClick = onAddContact) {
                        Icon(Icons.Default.Add, contentDescription = "Add", tint = OnDarkSecondary)
                    }
                    IconButton(onClick = onSettings) {
                        Icon(Icons.Default.Settings, contentDescription = "Settings", tint = OnDarkSecondary)
                    }
                },
            )
        },
    ) { padding ->
        if (state.conversations.isEmpty() && !state.isLoading) {
            androidx.compose.foundation.layout.Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "No conversations yet",
                    color = OnDarkSecondary,
                )
            }
        } else {
            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding),
            ) {
                items(state.conversations) { chat ->
                    ContactItem(
                        chat = chat,
                        onClick = { viewModel.onEvent(ChatListEvent.OpenChat(chat.contactId)) },
                    )
                }
            }
        }
    }
}
