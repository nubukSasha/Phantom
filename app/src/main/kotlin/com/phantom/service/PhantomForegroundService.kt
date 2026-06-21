package com.phantom.service

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.phantom.MainActivity
import com.phantom.PhantomApp
import com.phantom.R

class PhantomForegroundService : Service() {

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        startForeground(NOTIFICATION_ID, createNotification())
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_LOCK) {
            PhantomApp.instance.lock()
            stopSelf()
            return START_NOT_STICKY
        }
        return START_STICKY
    }

    private fun createNotification(): Notification {
        val openIntent = Intent(this, MainActivity::class.java)
        val openPending = PendingIntent.getActivity(
            this, 0, openIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val lockIntent = Intent(this, PhantomForegroundService::class.java).apply {
            action = ACTION_LOCK
        }
        val lockPending = PendingIntent.getService(
            this, 1, lockIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        return NotificationCompat.Builder(this, PhantomApp.CHANNEL_ID)
            .setContentTitle(getString(R.string.foreground_notification_title))
            .setContentText(getString(R.string.foreground_notification_text))
            .setSmallIcon(android.R.drawable.ic_lock_lock)
            .setContentIntent(openPending)
            .addAction(android.R.drawable.ic_lock_lock, getString(R.string.lock), lockPending)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }

    companion object {
        const val NOTIFICATION_ID = 1
        const val ACTION_LOCK = "com.phantom.action.LOCK"
    }
}
