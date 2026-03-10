package app.tauri.notification

import android.content.Context
import android.util.Log
import androidx.annotation.VisibleForTesting
import app.tauri.plugin.JSObject
import org.json.JSONObject
import org.unifiedpush.android.connector.FailedReason
import org.unifiedpush.android.connector.MessagingReceiver
import org.unifiedpush.android.connector.data.PushEndpoint
import org.unifiedpush.android.connector.data.PushMessage
import java.util.concurrent.Executor
import java.util.concurrent.Executors

/**
 * Generic UnifiedPush receiver that forwards messages to the JS layer
 * and optionally delegates to a custom [UnifiedPushMessageHandler].
 */
open class TauriUnifiedPushMessagingService : MessagingReceiver() {

  companion object {
    private const val TAG = "TauriUnifiedPush"
    private var executor: Executor = Executors.newSingleThreadExecutor()

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

    /**
     * Replace the executor used for running custom message handlers.
     * Intended for testing only — pass a direct/synchronous executor to avoid
     * flaky `Thread.sleep()` calls in tests.
     */
    @VisibleForTesting
    @JvmStatic
    internal fun setExecutorForTesting(testExecutor: Executor) {
      executor = testExecutor
    }
  }

  override fun onNewEndpoint(context: Context, endpoint: PushEndpoint, instance: String) {
    Log.d(TAG, "New endpoint registered: ${endpoint.url}")
    val pubKeySet = endpoint.pubKeySet
    NotificationPlugin.instance?.handleNewUnifiedPushEndpoint(
      endpoint.url,
      instance,
      pubKeySet?.pubKey,
      pubKeySet?.auth,
    )
  }

  override fun onUnregistered(context: Context, instance: String) {
    Log.d(TAG, "Unregistered for instance: $instance")
    NotificationPlugin.instance?.handleUnifiedPushUnregistered(instance)
  }

  /**
   * Called when the distributor is temporarily unavailable (e.g. the distributor app
   * is being updated). The registration remains valid; the app should wait for a new
   * [onNewEndpoint] callback before sending push messages.
   */
  override fun onTempUnavailable(context: Context, instance: String) {
    Log.d(TAG, "Temporarily unavailable for instance: $instance")
    NotificationPlugin.instance?.handleUnifiedPushTempUnavailable(instance)
  }

  override fun onMessage(context: Context, message: PushMessage, instance: String) {
    Log.d(TAG, "Message received for instance: $instance")

    try {
      val messageString = message.content.toString(Charsets.UTF_8)

      val pushData = mutableMapOf<String, Any>()
      try {
        val json = JSONObject(messageString)
        for (key in json.keys()) {
          pushData[key] = JSObjectUtils.jsonValueToNative(json.get(key))
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
            val handled = handler.onMessage(context, message.content, instance)
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
      JSObjectUtils.putValueToJSObject(extraData, key, value)
    }
    val notification = Notification().apply {
      id = (System.nanoTime() % Int.MAX_VALUE).toInt()
      this.title = title ?: ""
      this.body = body
      this.isAutoCancel = true
      this.extra = extraData
    }

    val plugin = NotificationPlugin.instance
    if (plugin != null) {
      plugin.getNotificationManager().schedule(notification, "unifiedpush")
    } else {
      Log.w(TAG, "NotificationPlugin not initialized, cannot show fallback notification")
    }
  }

  override fun onRegistrationFailed(context: Context, reason: FailedReason, instance: String) {
    Log.e(TAG, "Registration failed for instance: $instance (reason: $reason)")
    NotificationPlugin.instance?.handleUnifiedPushRegistrationFailed(instance, reason.toString())
  }
}
