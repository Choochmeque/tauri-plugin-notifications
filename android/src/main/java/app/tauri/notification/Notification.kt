package app.tauri.notification

import android.content.ContentResolver
import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Build
import android.service.notification.StatusBarNotification
import androidx.annotation.RequiresApi
import app.tauri.annotation.InvokeArg
import app.tauri.plugin.JSObject
import com.fasterxml.jackson.annotation.JsonProperty

@InvokeArg
class MessagingStylePerson {
  var name: String = ""
  var icon: String? = null
  var iconUrl: String? = null
  var key: String? = null
}

@InvokeArg
class MessagingStyleMessage {
  var text: String = ""
  var timestamp: Long = 0
  var sender: MessagingStylePerson? = null
}

@InvokeArg
class MessagingStyleConfig {
  var user: MessagingStylePerson = MessagingStylePerson()
  var conversationTitle: String? = null
  var isGroupConversation: Boolean = false
  var messages: List<MessagingStyleMessage> = listOf()
  @JsonProperty(access = JsonProperty.Access.WRITE_ONLY)
  var authToken: String? = null
}

@InvokeArg
class Notification {
  var id: Int = 0
  var title: String? = null
  var body: String? = null
  var largeBody: String? = null
  var summary: String? = null
  var sound: String? = null
  var icon: String? = null
  var largeIcon: String? = null
  var iconColor: String? = null
  var actionTypeId: String? = null
  var group: String? = null
  var inboxLines: List<String>? = null
  var isGroupSummary = false
  var isOngoing = false
  var isAutoCancel = false
  var extra: JSObject? = null
  var attachments: List<NotificationAttachment>? = null
  var schedule: NotificationSchedule? = null
  var channelId: String? = null
  var sourceJson: String? = null
  var visibility: Int? = null
  var number: Int? = null
  var silent: Boolean? = null

  // Progress bar support
  var progress: Int? = null
  var progressMax: Int? = null
  var progressIndeterminate: Boolean? = null

  // System category (maps to NotificationCompat.CATEGORY_* constants)
  var category: String? = null

  // MessagingStyle support
  var messagingStyle: MessagingStyleConfig? = null

  fun getSound(context: Context, defaultSound: Int): String? {
    var soundPath: String? = null
    var resId: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
    val name = AssetUtils.getResourceBaseName(sound)
    if (name != null) {
      resId = AssetUtils.getResourceID(context, name, "raw")
    }
    if (resId == AssetUtils.RESOURCE_ID_ZERO_VALUE) {
      resId = defaultSound
    }
    if (resId != AssetUtils.RESOURCE_ID_ZERO_VALUE) {
      soundPath =
        ContentResolver.SCHEME_ANDROID_RESOURCE + "://" + context.packageName + "/" + resId
    }
    return soundPath
  }

  fun getIconColor(globalColor: String): String {
    // use the one defined local before trying for a globally defined color
    return iconColor ?: globalColor
  }

  fun getSmallIcon(context: Context, defaultIcon: Int): Int {
    var resId: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
    if (icon != null) {
      resId = AssetUtils.getResourceID(context, icon, "drawable")
    }
    if (resId == AssetUtils.RESOURCE_ID_ZERO_VALUE) {
      resId = defaultIcon
    }
    return resId
  }

  fun getLargeIcon(context: Context): Bitmap? {
    if (largeIcon != null) {
      val resId: Int = AssetUtils.getResourceID(context, largeIcon, "drawable")
      return BitmapFactory.decodeResource(context.resources, resId)
    }
    return null
  }

  companion object {
    fun buildNotificationPendingList(notifications: List<Notification>): List<PendingNotification> {
      val pendingNotifications = mutableListOf<PendingNotification>()
      for (notification in notifications) {
        val pendingNotification = PendingNotification().apply {
          id = notification.id
          title = notification.title
          body = notification.body
          schedule = notification.schedule
          extra = notification.extra
        }
        pendingNotifications.add(pendingNotification)
      }
      return pendingNotifications
    }

    @RequiresApi(Build.VERSION_CODES.M)
    fun buildNotificationActiveList(statusBarNotifications: Array<StatusBarNotification>): List<ActiveNotificationInfo> {
      val activeNotifications = mutableListOf<ActiveNotificationInfo>()
      for (statusBarNotification in statusBarNotifications) {
        val notification = statusBarNotification.notification
        val extractedData = mutableMapOf<String, String>()
        if (notification != null) {
          for (key in notification.extras.keySet()) {
            notification.extras.getString(key)?.let { value ->
              extractedData[key] = value
            }
          }
        }

        val activeNotification = ActiveNotificationInfo().apply {
          id = statusBarNotification.id
          tag = statusBarNotification.tag
          title = notification?.extras?.getCharSequence(android.app.Notification.EXTRA_TITLE)?.toString()
          body = notification?.extras?.getCharSequence(android.app.Notification.EXTRA_TEXT)?.toString()
          group = notification?.group
          groupSummary = notification?.let { 0 != it.flags and android.app.Notification.FLAG_GROUP_SUMMARY } ?: false
          data = extractedData
        }
        activeNotifications.add(activeNotification)
      }
      return activeNotifications
    }
  }
}

@InvokeArg
class PendingNotification {
  var id: Int = 0
  var title: String? = null
  var body: String? = null
  var schedule: NotificationSchedule? = null
  var extra: JSObject? = null
}

@InvokeArg
class ActiveNotificationInfo {
  var id: Int = 0
  var tag: String? = null
  var title: String? = null
  var body: String? = null
  var group: String? = null
  var groupSummary: Boolean = false
  var data: Map<String, String> = emptyMap()
  var extra: Map<String, Any> = emptyMap()
  var attachments: List<AttachmentInfo> = emptyList()
  var actionTypeId: String? = null
  var schedule: NotificationSchedule? = null
  var sound: String? = null
}

@InvokeArg
class AttachmentInfo {
  var id: String = ""
  var url: String = ""
}