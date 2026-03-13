package dev.paw.android

import android.util.Log
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage

class PawFirebaseMessagingService : FirebaseMessagingService() {
    override fun onNewToken(token: String) {
        Log.d(TAG, "Received refreshed FCM token (${token.take(8)}…)")
    }

    override fun onMessageReceived(message: RemoteMessage) {
        Log.d(TAG, "Received FCM message with data keys=${message.data.keys}")
    }

    companion object {
        private const val TAG = "PawFirebaseMessaging"
    }
}
