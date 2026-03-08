package app.tauri.notification

import android.content.Context
import android.util.Log
import app.tauri.plugin.JSObject
import org.json.JSONArray
import org.json.JSONObject
import org.unifiedpush.android.connector.MessagingReceiver
import java.util.concurrent.Executors

/**
 * Generic UnifiedPush receiver that forwards messages to the JS layer
 * and optionally delegates to a custom [UnifiedPushMessageHandler].
 */
open class TauriUnifiedPushMessagingService : MessagingReceiver() {

  companion object {
    private const val TAG = "TauriUnifiedPush"
    private val executor = Executors.newSingleThreadExecutor()

    @Volatile
    private var messageHandler: UnifiedPushMessageHandler? = null

    /**
     * Register a custom handler for incoming UnifiedPush messages.
     * The handler runs on a background thread, so network I/O is safe.
     * If the handler returns `true`, the default fallback notification is suppressed.
     */
    @JvmStatic
    fun setMessageHandler(handler: UnifiedPushMessageHandler?) {
      messageHandler = handler
    }
  }

  override fun onNewEndpoint(context: Context, endpoint: String, instance: String) {
    Log.d(TAG, "New endpoint registered: $endpoint")
    NotificationPlugin.instance?.handleNewUnifiedPushEndpoint(endpoint, instance)
  }

  override fun onUnregistered(context: Context, instance: String) {
    Log.d(TAG, "Unregistered for instance: $instance")
    NotificationPlugin.instance?.handleUnifiedPushUnregistered(instance)
  }

  override fun onMessage(context: Context, message: ByteArray, instance: String) {
    Log.d(TAG, "Message received for instance: $instance")

    try {
      val messageString = message.toString(Charsets.UTF_8)

      val pushData = mutableMapOf<String, Any>()
      try {
        val json = JSONObject(messageString)
        for (key in json.keys()) {
          pushData[key] = jsonValueToNative(json.get(key))
        }
      } catch (e: Exception) {
        Log.w(TAG, "Message is not valid JSON, forwarding as raw text")
        pushData["body"] = messageString
      }

      pushData["instance"] = instance
      pushData["source"] = "unifiedpush"

      NotificationPlugin.instance?.triggerUnifiedPushMessage(pushData)

      val handler = messageHandler
      if (handler != null) {
        executor.execute {
          try {
            val handled = handler.onMessage(context, message, instance)
            if (!handled) showFallbackNotification(pushData)
          } catch (e: Exception) {
            Log.e(TAG, "Message handler error: ${e.message}", e)
            showFallbackNotification(pushData)
          }
        }
      } else {
        showFallbackNotification(pushData)
      }
    } catch (e: Exception) {
      Log.e(TAG, "Error processing message: ${e.message}", e)
    }
  }

  private fun showFallbackNotification(pushData: Map<String, Any>) {
    val title = pushData["title"]?.toString()
    val body = pushData["body"]?.toString()
    if (title == null && body == null) return

    val extraData = JSObject()
    for ((key, value) in pushData) {
      when (value) {
        is String -> extraData.put(key, value)
        is Int -> extraData.put(key, value)
        is Long -> extraData.put(key, value)
        is Double -> extraData.put(key, value)
        is Boolean -> extraData.put(key, value)
        else -> extraData.put(key, value.toString())
      }
    }
    val notification = Notification().apply {
      id = System.currentTimeMillis().toInt()
      this.title = title ?: ""
      this.body = body
      this.isAutoCancel = true
      this.extra = extraData
    }
    NotificationPlugin.triggerNotification(notification, "unifiedpush")
  }

  override fun onRegistrationFailed(context: Context, instance: String) {
    Log.e(TAG, "Registration failed for instance: $instance")
    NotificationPlugin.instance?.handleUnifiedPushRegistrationFailed(instance)
  }

  private fun jsonValueToNative(value: Any): Any {
    return when (value) {
      is JSONObject -> {
        val map = mutableMapOf<String, Any>()
        for (key in value.keys()) {
          map[key] = jsonValueToNative(value.get(key))
        }
        map
      }
      is JSONArray -> {
        val list = mutableListOf<Any>()
        for (i in 0 until value.length()) {
          list.add(jsonValueToNative(value.get(i)))
        }
        list
      }
      else -> value
    }
  }
}
