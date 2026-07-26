package app.tauri.notification

import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import org.json.JSONObject

class TauriFirebaseMessagingService : FirebaseMessagingService() {

  override fun onNewToken(token: String) {
    super.onNewToken(token)
    NotificationPlugin.instance?.handleNewToken(token)
  }

  override fun onMessageReceived(message: RemoteMessage) {
    super.onMessageReceived(message)

    val pushData = mutableMapOf<String, Any>()

    message.notification?.let { notification ->
      notification.title?.let { pushData["title"] = it }
      notification.body?.let { pushData["body"] = it }
      notification.channelId?.let { pushData["channelId"] = it }
      notification.sound?.let { pushData["sound"] = it }
      notification.tag?.let { pushData["tag"] = it }
    }

    if (message.data.isNotEmpty()) {
      pushData["data"] = message.data
    }

    message.messageId?.let { pushData["messageId"] = it }
    message.from?.let { pushData["from"] = it }
    pushData["sentTime"] = message.sentTime

    if (message.data.isNotEmpty() && UnifiedPushStateStore(this).activeProvider == "fcm") {
      val dataJson = JSONObject(message.data as Map<String, Any>)
      UnifiedPushNotifier.showFromPush(this, dataJson.toString())
    }

    NotificationPlugin.instance?.triggerPushMessage(pushData)
  }
}
