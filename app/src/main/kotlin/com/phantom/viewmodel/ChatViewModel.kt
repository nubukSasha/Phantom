package com.phantom.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.phantom.PhantomApp
import com.phantom.FfiDirection
import com.phantom.FfiMessage
import com.phantom.model.Message
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class ChatUiState(
    val contactId: Long,
    val contactAlias: String = "",
    val messages: List<Message> = emptyList(),
    val inputText: String = "",
    val isSending: Boolean = false,
    val isLoading: Boolean = true,
    val error: String? = null,
)

sealed interface ChatEvent {
    data class Load(val contactId: Long) : ChatEvent
    data class UpdateInput(val text: String) : ChatEvent
    data object Send : ChatEvent
    data object MarkSeen : ChatEvent
    data object NavigateBack : ChatEvent
}

class ChatViewModel : ViewModel() {

    private val _state = MutableStateFlow(ChatUiState(contactId = 0))
    val state: StateFlow<ChatUiState> = _state.asStateFlow()

    private val _navigation = MutableStateFlow<ChatEvent?>(null)
    val navigation: StateFlow<ChatEvent?> = _navigation.asStateFlow()

    fun onEvent(event: ChatEvent) {
        when (event) {
            is ChatEvent.Load -> loadMessages(event.contactId)
            is ChatEvent.UpdateInput -> _state.value = _state.value.copy(inputText = event.text)
            ChatEvent.Send -> sendMessage()
            ChatEvent.MarkSeen -> markSeen()
            ChatEvent.NavigateBack -> _navigation.value = event
        }
    }

    fun onNavigated() {
        _navigation.value = null
    }

    private fun loadMessages(contactId: Long) {
        viewModelScope.launch {
            _state.value = _state.value.copy(
                contactId = contactId, isLoading = true
            )
            try {
                val core = PhantomApp.instance.core
                val contact = core.getContact(contactId)
                val msgs = core.getMessages(contactId, limit = 50u, offset = 0u)
                _state.value = _state.value.copy(
                    contactAlias = contact.alias,
                    messages = msgs.map { it.toMessage() },
                    isLoading = false,
                )
                markSeen()
            } catch (e: Exception) {
                _state.value = _state.value.copy(
                    isLoading = false, error = e.message
                )
            }
        }
    }

    private fun sendMessage() {
        val text = _state.value.inputText.trim()
        if (text.isEmpty() || _state.value.isSending) return

        viewModelScope.launch {
            _state.value = _state.value.copy(isSending = true, error = null)
            try {
                val msgId = PhantomApp.instance.core.sendMessage(
                    _state.value.contactId, text
                )
                _state.value = _state.value.copy(
                    inputText = "", isSending = false
                )
                loadMessages(_state.value.contactId)
            } catch (e: Exception) {
                _state.value = _state.value.copy(
                    isSending = false, error = e.message
                )
            }
        }
    }

    private fun markSeen() {
        viewModelScope.launch {
            try {
                PhantomApp.instance.core.markSeen(_state.value.contactId)
            } catch (_: Exception) { }
        }
    }

    companion object {
        private fun FfiMessage.toMessage() = Message(
            messageId = id,
            contactId = contactId,
            text = text ?: "",
            direction = when (direction) {
                FfiDirection.SENT -> com.phantom.model.Direction.Sent
                FfiDirection.RECEIVED -> com.phantom.model.Direction.Received
                else -> com.phantom.model.Direction.Received
            },
            createdAt = createdAt,
            deliveredAt = deliveredAt,
            seenAt = seenAt,
        )
    }
}
