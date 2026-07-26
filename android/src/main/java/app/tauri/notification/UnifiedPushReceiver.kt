package app.tauri.notification

import android.content.Context
import org.unifiedpush.android.connector.FailedReason
import org.unifiedpush.android.connector.MessagingReceiver
import org.unifiedpush.android.connector.UnifiedPush
import org.unifiedpush.android.connector.data.PushEndpoint
import org.unifiedpush.android.connector.data.PushMessage
import org.unifiedpush.android.connector.keys.KeyManager

class UnifiedPushReceiver : MessagingReceiver() {

    override fun getKeyManager(context: Context): KeyManager {
        return CachedKeyManager.getInstance(context)
    }

    override fun onNewEndpoint(context: Context, endpoint: PushEndpoint, instance: String) {
        NotificationPlugin.instance?.onUnifiedPushNewEndpoint(
            endpoint.url,
            endpoint.pubKeySet?.pubKey,
            endpoint.pubKeySet?.auth,
            instance,
        )
    }

    override fun onRegistrationFailed(context: Context, reason: FailedReason, instance: String) {
        NotificationPlugin.instance?.onUnifiedPushRegistrationFailed(reason.name, instance)
    }

    override fun onUnregistered(context: Context, instance: String) {
        NotificationPlugin.instance?.onUnifiedPushUnregistered(instance)
    }

    override fun onTempUnavailable(context: Context, instance: String) {
        NotificationPlugin.instance?.onUnifiedPushTemporaryUnavailable(instance)
    }

    override fun onMessage(context: Context, message: PushMessage, instance: String) {
        val content = String(message.content, Charsets.UTF_8)
        val state = UnifiedPushStateStore(context)
        if (instance != state.activeInstance || state.activeProvider != "unifiedpush") return
        // Always show the native notification immediately from the push payload.
        // This eliminates the JS round-trip delay on the warm path (app alive in
        // background). JS still receives the push-message event for in-app badge
        // updates and notification enrichment (inbox grouping, fetched content
        // for event_id_only payloads). When JS calls sendNotification() with the
        // same notification ID, Android UPDATES the existing notification rather
        // than showing a duplicate.
        UnifiedPushNotifier.showFromPush(context, content)
        NotificationPlugin.instance?.onUnifiedPushMessage(content, instance)
    }
}
