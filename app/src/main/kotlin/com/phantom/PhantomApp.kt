package com.phantom

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import android.os.Build
import com.phantom.bridge.AndroidKeystoreCallback

class PhantomApp : Application() {

    lateinit var core: PhantomCore
        private set

    private val keystore = AndroidKeystoreCallback()
    private var messageCallback: PhantomMessageCallback? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        instance = this
    }

    suspend fun initCore() {
        if (::core.isInitialized) return
        val storagePath = filesDir.resolve("phantom.db").absolutePath
        messageCallback = PhantomMessageCallback()
        core = PhantomCore.init(
            storagePath = storagePath,
            keystore = keystore,
            callback = messageCallback!!,
        )
    }

    fun lock() {
        if (::core.isInitialized) core.lock()
    }

    fun unlock() {
        if (::core.isInitialized) core.unlock(keystore = keystore)
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val ch = NotificationChannel(
                CHANNEL_ID,
                getString(R.string.channel_name),
                NotificationManager.IMPORTANCE_HIGH
            )
            ch.description = getString(R.string.channel_desc)
            val nm = getSystemService(NotificationManager::class.java)
            nm.createNotificationChannel(ch)
        }
    }

    class PhantomMessageCallback : MessageCallback {
        override fun onMessageReceived(contactId: Long, text: String, messageId: Long) {
            // Posted to ViewModel via shared flow
            _messageFlow.trySend(
                IncomingMessage(contactId, text, messageId)
            )
        }

        override fun onContactOnline(contactId: Long) {
            _contactStatusFlow.trySend(
                ContactStatus(contactId, true)
            )
        }

        override fun onContactOffline(contactId: Long) {
            _contactStatusFlow.trySend(
                ContactStatus(contactId, false)
            )
        }
    }

    companion object {
        const val CHANNEL_ID = "phantom_messages"
        lateinit var instance: PhantomApp
            private set

        private val _messageFlow = kotlinx.coroutines.channels.Channel<IncomingMessage>(
            capacity = kotlinx.coroutines.channels.Channel.BUFFERED
        )
        val messageFlow = _messageFlow.receiveAsFlow()

        private val _contactStatusFlow = kotlinx.coroutines.channels.Channel<ContactStatus>(
            capacity = kotlinx.coroutines.channels.Channel.BUFFERED
        )
        val contactStatusFlow = _contactStatusFlow.receiveAsFlow()

        data class IncomingMessage(
            val contactId: Long,
            val text: String,
            val messageId: Long,
        )

        data class ContactStatus(
            val contactId: Long,
            val isOnline: Boolean,
        )
    }
}
