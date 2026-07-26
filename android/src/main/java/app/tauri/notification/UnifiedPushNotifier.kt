package app.tauri.notification

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.RemoteInput
import com.fasterxml.jackson.databind.ObjectMapper
import org.json.JSONObject

object UnifiedPushNotifier {
    private const val CHANNEL_ID = "messages"
    private const val ACTION_TYPE_ID = "sable-message"
    private const val GROUP_KEY = "matrix_messages"

    fun showFromPush(context: Context, rawMessage: String) {
        val rootJson = try {
            JSONObject(rawMessage)
        } catch (e: Exception) {
            null
        } ?: return

        // Also accept the payload nested as a JSON string, not just an object.
        val notification = rootJson.optJSONObject("notification")
            ?: rootJson.optString("notification").takeIf { it.isNotEmpty() }?.let {
                try { JSONObject(it) } catch (e: Exception) { null }
            }
            ?: return

        val roomId = notification.optString("room_id")
        val eventId = notification.optString("event_id")
        val sender = notification.optString("sender_display_name")
        val title = notification.optString("room_name").ifEmpty {
            notification.optString("sender_display_name").ifEmpty { "New message" }
        }
        val body = buildBody(notification, sender)

        val userId = rootJson.optString("user_id").ifEmpty {
            notification.optString("user_id")
        }

        ensureChannel(context)

        val iconId = context.resources
            .getIdentifier("notification_icon", "drawable", context.packageName)
            .takeIf { it != 0 } ?: android.R.drawable.ic_dialog_info

        val notifId = sableNotifId(userId, roomId)

        val intent = buildPushIntent(context, notifId, roomId, eventId, userId)

        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            PendingIntent.FLAG_CANCEL_CURRENT or PendingIntent.FLAG_MUTABLE
        } else {
            PendingIntent.FLAG_CANCEL_CURRENT
        }

        val builder = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(iconId)
            .setContentTitle(title)
            .setContentText(body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(body))
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setGroup(GROUP_KEY)
            .setContentIntent(
                PendingIntent.getActivity(context, notifId, intent, flags)
            )

        addReplyAction(context, builder, notifId, roomId, eventId, userId, flags)

        NotificationManagerCompat.from(context).notify(notifId, builder.build())
    }

    private fun addReplyAction(
        context: Context,
        builder: NotificationCompat.Builder,
        notifId: Int,
        roomId: String,
        eventId: String,
        userId: String,
        flags: Int
    ) {
        val storage = NotificationStorage(context, ObjectMapper())
        val actions = storage.getActionGroup(ACTION_TYPE_ID)
        for (action in actions) {
            if (action == null) continue
            val actionIntent = buildPushIntent(context, notifId, roomId, eventId, userId, action.id)
            val actionPendingIntent = PendingIntent.getActivity(
                context, notifId + action.id.hashCode(), actionIntent, flags
            )
            val actionBuilder = NotificationCompat.Action.Builder(
                R.drawable.ic_transparent, action.title, actionPendingIntent
            )
            if (action.input == true) {
                actionBuilder.addRemoteInput(
                    RemoteInput.Builder(REMOTE_INPUT_KEY).setLabel(action.title).build()
                )
            }
            builder.addAction(actionBuilder.build())
        }
    }

    private fun sableNotifId(userId: String, roomId: String): Int {
        val key = "$userId\u0000$roomId"
        var hash = 0
        for (element in key) {
            hash = 31 * hash + element.code
        }
        return Math.abs(hash)
    }

    private fun buildPushIntent(
        context: Context,
        notifId: Int,
        roomId: String,
        eventId: String,
        userId: String,
        action: String = DEFAULT_PRESS_ACTION
    ): Intent {
        val intent = context.packageManager.getLaunchIntentForPackage(context.packageName)
            ?: Intent(Intent.ACTION_MAIN).setPackage(context.packageName)
        intent.action = Intent.ACTION_MAIN
        intent.addCategory(Intent.CATEGORY_LAUNCHER)
        intent.flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
        intent.putExtra(NOTIFICATION_INTENT_KEY, notifId)
        intent.putExtra(ACTION_INTENT_KEY, action)
        intent.putExtra(NOTIFICATION_IS_REMOVABLE_KEY, true)

        val extraJson = JSONObject().apply {
            put("room_id", roomId)
            put("event_id", eventId)
            if (userId.isNotEmpty()) put("user_id", userId)
            put("instance", UnifiedPushStateStore.INSTANCE)
        }
        val sourceJson = JSONObject().apply {
            put("id", notifId)
            put("extra", extraJson)
            put("actionTypeId", ACTION_TYPE_ID)
        }.toString()
        intent.putExtra(NOTIFICATION_OBJ_INTENT_KEY, sourceJson)

        return intent
    }

    private fun buildBody(notification: JSONObject, sender: String): String {
        val prefix = if (sender.isNotEmpty()) "$sender: " else ""
        if (notification.optString("type") == "m.room.encrypted") {
            return "${prefix}Encrypted message"
        }
        val text = notification.optJSONObject("content")?.optString("body")?.takeIf { it.isNotEmpty() }
        return if (text != null) "$prefix$text" else "${prefix}New message"
    }

    private fun ensureChannel(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        if (manager.getNotificationChannel(CHANNEL_ID) == null) {
            manager.createNotificationChannel(
                NotificationChannel(CHANNEL_ID, "Messages", NotificationManager.IMPORTANCE_HIGH).apply {
                    description = "Matrix message and invite notifications"
                }
            )
        }
    }
}
