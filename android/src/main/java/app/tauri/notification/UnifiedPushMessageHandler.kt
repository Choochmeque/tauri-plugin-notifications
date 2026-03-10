package app.tauri.notification

import android.content.Context

/**
 * Interface for custom handling of incoming UnifiedPush messages.
 *
 * Register via [TauriUnifiedPushMessagingService.setMessageHandler].
 * Implementations run on a background thread — network I/O is safe.
 */
interface UnifiedPushMessageHandler {
  /**
   * @param context  application context
   * @param message  raw push payload as bytes
   * @param instance UnifiedPush registration instance identifier
   * @return `true` if handled (suppresses the default notification), `false` for fallback
   */
  fun onMessage(context: Context, message: ByteArray, instance: String): Boolean
}

