package com.phantom.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.phantom.PhantomApp
import com.phantom.FfiConversationSummary
import com.phantom.model.Chat
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class ChatListUiState(
    val conversations: List<Chat> = emptyList(),
    val isLoading: Boolean = true,
    val error: String? = null,
)

sealed interface ChatListEvent {
    data object Refresh : ChatListEvent
    data class OpenChat(val contactId: Long) : ChatListEvent
    data object NavigateToAddContact : ChatListEvent
    data object NavigateToSettings : ChatListEvent
    data class DeleteConversation(val contactId: Long) : ChatListEvent
}

class ChatListViewModel : ViewModel() {

    private val _state = MutableStateFlow(ChatListUiState())
    val state: StateFlow<ChatListUiState> = _state.asStateFlow()

    private val _navigation = MutableStateFlow<ChatListEvent?>(null)
    val navigation: StateFlow<ChatListEvent?> = _navigation.asStateFlow()

    init {
        loadConversations()
        observeIncomingMessages()
    }

    fun onEvent(event: ChatListEvent) {
        when (event) {
            ChatListEvent.Refresh -> loadConversations()
            ChatListEvent.NavigateToAddContact -> _navigation.value = event
            ChatListEvent.NavigateToSettings -> _navigation.value = event
            is ChatListEvent.OpenChat -> _navigation.value = event
            is ChatListEvent.DeleteConversation -> deleteConversation(event.contactId)
        }
    }

    fun onNavigated() {
        _navigation.value = null
    }

    private fun loadConversations() {
        viewModelScope.launch {
            _state.value = _state.value.copy(isLoading = true)
            try {
                val core = PhantomApp.instance.core
                val convos = core.getConversations()
                _state.value = _state.value.copy(
                    conversations = convos.map { it.toChat() },
                    isLoading = false,
                )
            } catch (e: Exception) {
                _state.value = _state.value.copy(
                    isLoading = false,
                    error = e.message,
                )
            }
        }
    }

    private fun deleteConversation(contactId: Long) {
        viewModelScope.launch {
            try {
                PhantomApp.instance.core.deleteContact(contactId)
                loadConversations()
            } catch (e: Exception) {
                _state.value = _state.value.copy(error = e.message)
            }
        }
    }

    private fun observeIncomingMessages() {
        viewModelScope.launch {
            PhantomApp.messageFlow.collect { incoming ->
                loadConversations()
            }
        }
    }
}

private fun FfiConversationSummary.toChat() = Chat(
    contactId = contactId,
    alias = alias,
    onionAddress = onionAddress,
    lastMessage = lastMessagePreview,
    lastMessageAt = lastMessageAt,
    unreadCount = unreadCount,
)
