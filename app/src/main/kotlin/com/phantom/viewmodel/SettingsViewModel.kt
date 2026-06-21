package com.phantom.viewmodel

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.phantom.PhantomApp
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class SettingsUiState(
    val identityKeyHex: String = "",
    val onionAddress: String = "",
    val isLocked: Boolean = false,
    val showDestroyConfirm: Boolean = false,
    val error: String? = null,
)

sealed interface SettingsEvent {
    data object CopyIdentityKey : SettingsEvent
    data object CopyOnionAddress : SettingsEvent
    data object ToggleLock : SettingsEvent
    data object ShowDestroyConfirm : SettingsEvent
    data object DismissDestroyConfirm : SettingsEvent
    data object ConfirmDestroy : SettingsEvent
    data object NavigateBack : SettingsEvent
}

class SettingsViewModel : ViewModel() {

    private val _state = MutableStateFlow(SettingsUiState())
    val state: StateFlow<SettingsUiState> = _state.asStateFlow()

    private val _navigation = MutableStateFlow<SettingsEvent?>(null)
    val navigation: StateFlow<SettingsEvent?> = _navigation.asStateFlow()

    init {
        loadSettings()
    }

    fun onEvent(event: SettingsEvent) {
        when (event) {
            SettingsEvent.CopyIdentityKey -> copyToClipboard(_state.value.identityKeyHex)
            SettingsEvent.CopyOnionAddress -> copyToClipboard(_state.value.onionAddress)
            SettingsEvent.ToggleLock -> toggleLock()
            SettingsEvent.ShowDestroyConfirm -> _state.value = _state.value.copy(showDestroyConfirm = true)
            SettingsEvent.DismissDestroyConfirm -> _state.value = _state.value.copy(showDestroyConfirm = false)
            SettingsEvent.ConfirmDestroy -> destroyAllData()
            SettingsEvent.NavigateBack -> _navigation.value = event
        }
    }

    fun onNavigated() {
        _navigation.value = null
    }

    private fun loadSettings() {
        val core = PhantomApp.instance.core
        _state.value = _state.value.copy(
            identityKeyHex = core.identityPublic().joinToString("") { "%02x".format(it) },
            onionAddress = core.onionAddress(),
        )
    }

    private fun toggleLock() {
        viewModelScope.launch {
            try {
                if (_state.value.isLocked) {
                    PhantomApp.instance.unlock()
                } else {
                    PhantomApp.instance.lock()
                }
                _state.value = _state.value.copy(isLocked = !_state.value.isLocked)
            } catch (e: Exception) {
                _state.value = _state.value.copy(error = e.message)
            }
        }
    }

    private fun destroyAllData() {
        viewModelScope.launch {
            try {
                PhantomApp.instance.core.destroy()
                _state.value = _state.value.copy(showDestroyConfirm = false)
                _navigation.value = SettingsEvent.NavigateBack
            } catch (e: Exception) {
                _state.value = _state.value.copy(
                    error = e.message, showDestroyConfirm = false
                )
            }
        }
    }

    private fun copyToClipboard(text: String) {
        val ctx = PhantomApp.instance
        val cm = ctx.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        cm.setPrimaryClip(ClipData.newPlainText("phantom", text))
    }
}
