// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.webkit.WebView
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

const val LOCAL_NOTIFICATIONS = "permissionState"

@InvokeArg
class PluginConfig {
  var icon: String? = null
  var sound: String? = null
  var iconColor: String? = null
}

@InvokeArg
class BatchArgs {
  lateinit var notifications: List<Notification>
}

@InvokeArg
class CancelArgs {
  lateinit var notifications: List<Int>
}

@InvokeArg
class NotificationAction {
  lateinit var id: String
  var title: String? = null
  var input: Boolean? = null
}

@InvokeArg
class ActionType {
  lateinit var id: String
  lateinit var actions: List<NotificationAction>
}

@InvokeArg
class RegisterActionTypesArgs {
  lateinit var types: List<ActionType>
}

@InvokeArg
class ActiveNotification {
  var id: Int = 0
  var tag: String? = null
}

@InvokeArg
class RemoveActiveArgs {
  var notifications: List<ActiveNotification> = listOf()
}

@TauriPlugin(
  permissions = [
    Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = "permissionState")
  ]
)
class NotificationPlugin(private val activity: Activity): Plugin(activity) {
  private var webView: WebView? = null
  private lateinit var manager: TauriNotificationManager
  private lateinit var notificationManager: NotificationManager
  private lateinit var notificationStorage: NotificationStorage
  private var channelManager = ChannelManager(activity)

  private var pendingTokenInvoke: Invoke? = null
  private var cachedToken: String? = null

  companion object {
    var instance: NotificationPlugin? = null

    fun triggerNotification(notification: Notification) {
      instance?.triggerObject("notification", notification)
    }
  }

  override fun load(webView: WebView) {
    instance = this

    super.load(webView)
    this.webView = webView
    notificationStorage = NotificationStorage(activity, jsonMapper())
    
    val manager = TauriNotificationManager(
      notificationStorage,
      activity,
      activity,
      getConfig(PluginConfig::class.java)
    )
    manager.createNotificationChannel()
    
    this.manager = manager
    
    notificationManager = activity.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

    val intent = activity.intent
    intent?.let {
      onIntent(it)
    }
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    onIntent(intent)
  }

  fun onIntent(intent: Intent) {
    if (Intent.ACTION_MAIN != intent.action) {
      return
    }
    val dataJson = manager.handleNotificationActionPerformed(intent, notificationStorage)
    if (dataJson != null) {
      trigger("actionPerformed", dataJson)
    }
  }

  @Command
  fun show(invoke: Invoke) {
    val notification = invoke.parseArgs(Notification::class.java)
    val id = manager.schedule(notification)

    invoke.resolveObject(id)
  }

  @Command
  fun batch(invoke: Invoke) {
    val args = invoke.parseArgs(BatchArgs::class.java)

    val ids = manager.schedule(args.notifications)
    notificationStorage.appendNotifications(args.notifications)

    invoke.resolveObject(ids)
  }

  @Command
  fun cancel(invoke: Invoke) {
    val args = invoke.parseArgs(CancelArgs::class.java)
    manager.cancel(args.notifications)
    invoke.resolve()
  }

  @Command
  fun removeActive(invoke: Invoke) {
    val args = invoke.parseArgs(RemoveActiveArgs::class.java)

    if (args.notifications.isEmpty()) {
      notificationManager.cancelAll()
      invoke.resolve()
    } else {
      for (notification in args.notifications) {
        if (notification.tag == null) {
          notificationManager.cancel(notification.id)
        } else {
          notificationManager.cancel(notification.tag, notification.id)
        }
      }
      invoke.resolve()
    }
  }

  @Command
  fun getPending(invoke: Invoke) {
    val notifications= notificationStorage.getSavedNotifications()
    val result = Notification.buildNotificationPendingList(notifications)
    invoke.resolveObject(result)
  }

  @Command
  fun registerActionTypes(invoke: Invoke) {
    val args = invoke.parseArgs(RegisterActionTypesArgs::class.java)
    notificationStorage.writeActionGroup(args.types)
    invoke.resolve()
  }

  @SuppressLint("ObsoleteSdkInt")
  @Command
  fun getActive(invoke: Invoke) {
    val notifications = JSArray()
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      val activeNotifications = notificationManager.activeNotifications
      for (activeNotification in activeNotifications) {
        val jsNotification = JSObject()
        jsNotification.put("id", activeNotification.id)
        jsNotification.put("tag", activeNotification.tag)
        val notification = activeNotification.notification
        if (notification != null) {
          jsNotification.put("title", notification.extras.getCharSequence(android.app.Notification.EXTRA_TITLE))
          jsNotification.put("body", notification.extras.getCharSequence(android.app.Notification.EXTRA_TEXT))
          jsNotification.put("group", notification.group)
          jsNotification.put(
            "groupSummary",
            0 != notification.flags and android.app.Notification.FLAG_GROUP_SUMMARY
          )
          val extras = JSObject()
          for (key in notification.extras.keySet()) {
            extras.put(key!!, notification.extras.getString(key))
          }
          jsNotification.put("data", extras)
        }
        notifications.put(jsNotification)
      }
    }
    
    invoke.resolveObject(notifications)
  }

  @Command
  fun createChannel(invoke: Invoke) {
    channelManager.createChannel(invoke)
  }

  @Command
  fun deleteChannel(invoke: Invoke) {
    channelManager.deleteChannel(invoke)
  }

  @Command
  fun listChannels(invoke: Invoke) {
    channelManager.listChannels(invoke)
  }

  @Command
  override fun checkPermissions(invoke: Invoke) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
      val permissionsResultJSON = JSObject()
      permissionsResultJSON.put("permissionState", getPermissionState())
      invoke.resolve(permissionsResultJSON)
    } else {
      super.checkPermissions(invoke)
    }
  }

  @Command
  override fun requestPermissions(invoke: Invoke) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
      permissionState(invoke)
    } else {
      if (getPermissionState(LOCAL_NOTIFICATIONS) !== PermissionState.GRANTED) {
        requestPermissionForAlias(LOCAL_NOTIFICATIONS, invoke, "permissionsCallback")
      }
    }
  }

  @Command
  fun registerForPushNotifications(invoke: Invoke) {
    if (!BuildConfig.ENABLE_PUSH_NOTIFICATIONS) {
      invoke.reject("Push notifications are disabled in this build")
      return
    }

    // First check if notifications are enabled
    if (!manager.areNotificationsEnabled()) {
      // Request permissions first
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        if (getPermissionState(LOCAL_NOTIFICATIONS) !== PermissionState.GRANTED) {
          // Request permissions and then get token
          pendingTokenInvoke = invoke
          requestPermissionForAlias(LOCAL_NOTIFICATIONS, invoke, "pushPermissionsCallback")
          return
        }
      } else {
        invoke.reject("Notification permissions not granted")
        return
      }
    }

    // If we already have a cached token, return it immediately
    cachedToken?.let {
      val result = JSObject()
      result.put("deviceToken", it)
      invoke.resolve(result)
      return
    }

    // Store the invoke to respond later when we get the token
    pendingTokenInvoke = invoke

    // Request the FCM token
    getFirebaseToken()
  }

  @PermissionCallback
  private fun pushPermissionsCallback(invoke: Invoke) {
    if (!manager.areNotificationsEnabled()) {
      invoke.reject("Notification permissions denied")
      pendingTokenInvoke = null
      return
    }

    // Permissions granted, now get the token
    getFirebaseToken()
  }

  private fun getFirebaseToken() {
    if (!BuildConfig.ENABLE_PUSH_NOTIFICATIONS) {
      pendingTokenInvoke?.reject("Push notifications are disabled in this build")
      pendingTokenInvoke = null
      return
    }

    try {
      val firebaseMessaging = Class.forName("com.google.firebase.messaging.FirebaseMessaging")
      val getInstance = firebaseMessaging.getMethod("getInstance")
      val instance = getInstance.invoke(null)
      val getToken = instance.javaClass.getMethod("getToken")
      val task = getToken.invoke(instance) as com.google.android.gms.tasks.Task<*>

      task.addOnCompleteListener { completedTask ->
        if (!completedTask.isSuccessful) {
          val errorMessage = "Failed to get FCM token: ${completedTask.exception?.message}"
          val errorData = JSObject()
          errorData.put("message", errorMessage)
          trigger("push-error", errorData)
          pendingTokenInvoke?.reject(errorMessage)
          pendingTokenInvoke = null
          return@addOnCompleteListener
        }

        val token = completedTask.result as String
        cachedToken = token
        val result = JSObject()
        result.put("deviceToken", token)
        pendingTokenInvoke?.resolve(result)
        pendingTokenInvoke = null
      }
    } catch (e: Exception) {
      val errorMessage = "Firebase not available: ${e.message}"
      val errorData = JSObject()
      errorData.put("message", errorMessage)
      trigger("push-error", errorData)
      pendingTokenInvoke?.reject(errorMessage)
      pendingTokenInvoke = null
    }
  }

  // Called by TauriFirebaseMessagingService when a new token is received
  fun handleNewToken(token: String) {
    if (!BuildConfig.ENABLE_PUSH_NOTIFICATIONS) return

    cachedToken = token
    // Trigger push-token event to notify the frontend about the token
    val data = JSObject()
    data.put("token", token)
    trigger("push-token", data)
  }

  // Called by TauriFirebaseMessagingService when a push message is received
  fun triggerPushMessage(pushData: Map<String, Any>) {
    if (!BuildConfig.ENABLE_PUSH_NOTIFICATIONS) return

    val data = JSObject()
    for ((key, value) in pushData) {
      when (value) {
        is String -> data.put(key, value)
        is Int -> data.put(key, value)
        is Long -> data.put(key, value)
        is Double -> data.put(key, value)
        is Boolean -> data.put(key, value)
        is Map<*, *> -> {
          val nestedObj = JSObject()
          @Suppress("UNCHECKED_CAST")
          val map = value as Map<String, Any>
          for ((k, v) in map) {
            nestedObj.put(k, v.toString())
          }
          data.put(key, nestedObj)
        }
        else -> data.put(key, value.toString())
      }
    }
    trigger("push-message", data)
  }

  @Command
  fun permissionState(invoke: Invoke) {
    val permissionsResultJSON = JSObject()
    permissionsResultJSON.put("permissionState", getPermissionState())
    invoke.resolve(permissionsResultJSON)
  }

  @PermissionCallback
  private fun permissionsCallback(invoke: Invoke) {
    val permissionsResultJSON = JSObject()
    permissionsResultJSON.put("permissionState", getPermissionState())
    invoke.resolve(permissionsResultJSON)
  }

  private fun getPermissionState(): String {
    return if (manager.areNotificationsEnabled()) {
      "granted"
    } else {
      "denied"
    }
  }
}
