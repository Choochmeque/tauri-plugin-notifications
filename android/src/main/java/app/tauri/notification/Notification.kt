// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
        val pendingNotification = PendingNotification(
          id = notification.id,
          title = notification.title,
          body = notification.body,
          schedule = notification.schedule,
          extra = notification.extra
        )
        pendingNotifications.add(pendingNotification)
      }
      return pendingNotifications
    }

    @RequiresApi(Build.VERSION_CODES.M)
    fun buildNotificationActiveList(statusBarNotifications: Array<StatusBarNotification>): List<ActiveNotificationInfo> {
      val activeNotifications = mutableListOf<ActiveNotificationInfo>()
      for (statusBarNotification in statusBarNotifications) {
        val notification = statusBarNotification.notification
        val data = mutableMapOf<String, String>()
        if (notification != null) {
          for (key in notification.extras.keySet()) {
            notification.extras.getString(key)?.let { value ->
              data[key] = value
            }
          }
        }

        val activeNotification = ActiveNotificationInfo(
          id = statusBarNotification.id,
          tag = statusBarNotification.tag,
          title = notification?.extras?.getCharSequence(android.app.Notification.EXTRA_TITLE)?.toString(),
          body = notification?.extras?.getCharSequence(android.app.Notification.EXTRA_TEXT)?.toString(),
          group = notification?.group,
          groupSummary = notification?.let { 0 != it.flags and android.app.Notification.FLAG_GROUP_SUMMARY } ?: false,
          data = data,
          extra = emptyMap(),
          attachments = emptyList(),
          actionTypeId = null,
          schedule = null,
          sound = null
        )
        activeNotifications.add(activeNotification)
      }
      return activeNotifications
    }
  }
}

class PendingNotification(
  val id: Int,
  val title: String?,
  val body: String?,
  val schedule: NotificationSchedule?,
  val extra: JSObject?
)

class ActiveNotificationInfo(
  val id: Int,
  val tag: String?,
  val title: String?,
  val body: String?,
  val group: String?,
  val groupSummary: Boolean,
  val data: Map<String, String>,
  val extra: Map<String, Any>,
  val attachments: List<AttachmentInfo>,
  val actionTypeId: String?,
  val schedule: NotificationSchedule?,
  val sound: String?
)

class AttachmentInfo(
  val id: String,
  val url: String
)