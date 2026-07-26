package app.tauri.notification

import android.content.Context
import org.unifiedpush.android.connector.UnifiedPush
import java.util.UUID

internal class UnifiedPushStateStore(private val context: Context) {
  private val prefs = context.getSharedPreferences("tauri-notifications", Context.MODE_PRIVATE)

  var activeProvider: String?
    get() = prefs.getString("push-provider", null)?.takeUnless { it == "none" }
      ?: if (!prefs.contains("push-provider") && UnifiedPush.getSavedDistributor(context) != null) "unifiedpush" else null
    set(value) = prefs.edit().putString("push-provider", value ?: "none").apply()
  var activeInstance: String?
    get() = prefs.getString("push-instance", null)
      ?: if (activeProvider == "unifiedpush") INSTANCE else null
    set(value) = prefs.edit().putString("push-instance", value ?: INSTANCE).apply()
  var endpoint: String?
    get() = prefs.getString("up-endpoint", null)
    set(value) = prefs.edit().putString("up-endpoint", value).apply()
  var p256dh: String?
    get() = prefs.getString("up-p256dh", null)
    set(value) = prefs.edit().putString("up-p256dh", value).apply()
  var auth: String?
    get() = prefs.getString("up-auth", null)
    set(value) = prefs.edit().putString("up-auth", value).apply()
  var distributor: String?
    get() = prefs.getString("up-distributor", null)
    set(value) = prefs.edit().putString("up-distributor", value).apply()
  var vapid: String?
    get() = prefs.getString("up-vapid", null)
    set(value) = prefs.edit().putString("up-vapid", value).apply()

  fun clearRegistration() {
    prefs.edit()
      .remove("push-instance")
      .remove("up-endpoint")
      .remove("up-p256dh")
      .remove("up-auth")
      .remove("up-distributor")
      .remove("up-vapid")
      .apply()
  }
  fun instanceForRegistration(): String {
    val current = activeInstance
    if (current != null && current != INSTANCE) {
      try { UnifiedPush.unregister(context, current, CachedKeyManager.getInstance(context)) } catch (_: Exception) {}
      activeInstance = INSTANCE
    }
    return INSTANCE
  }
  fun ensureExplicitInstance() {
    if (prefs.getString("push-instance", null) == null) {
      activeInstance = INSTANCE
    }
  }
  fun setUnifiedPushActive() { activeProvider = "unifiedpush" }

  companion object {
    const val INSTANCE = "default"
  }
}
