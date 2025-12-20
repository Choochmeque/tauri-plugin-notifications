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
import app.tauri.Logger
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
class SetClickListenerActiveArgs {
  var active: Boolean = false
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

  // Click listener tracking for cold-start support
  private var hasClickedListener = false
  private var pendingNotificationClick: JSObject? = null

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
    Logger.debug(Logger.tags(TAG), "onIntent called - action: ${intent.action}, extras: ${intent.extras?.keySet()}")

    // Handle local notification click (requires ACTION_MAIN)
    if (Intent.ACTION_MAIN == intent.action) {
      val dataJson = manager.handleNotificationActionPerformed(intent, notificationStorage)
      if (dataJson != null) {
        trigger("actionPerformed", dataJson)
        triggerNotificationClicked(
          intent.getIntExtra(NOTIFICATION_INTENT_KEY, -1),
          extractLocalNotificationData(intent)
        )
        return
      }
    }

    // Handle push notification click (Firebase background notification)
    // Firebase may use different actions, so check for push data regardless of action
    val pushData = extractPushNotificationData(intent)
    if (pushData != null) {
      Logger.debug(Logger.tags(TAG), "Push notification clicked with data: $pushData")
      triggerNotificationClicked(-1, pushData)
    }
  }

  private fun extractLocalNotificationData(intent: Intent): JSObject? {
    val notificationJson = intent.getStringExtra(NOTIFICATION_OBJ_INTENT_KEY) ?: return null
    return try {
      val notification = JSObject(notificationJson)
      if (notification.has("extra")) notification.getJSObject("extra") else null
    } catch (e: Exception) {
      Logger.error(Logger.tags(TAG), "Failed to extract local notification data: ${e.message}", e)
      null
    }
  }

  private fun extractPushNotificationData(intent: Intent): JSObject? {
    val extras = intent.extras ?: return null
    // Skip if no extras or if it's a regular app launch
    if (extras.isEmpty) return null

    Logger.debug(Logger.tags(TAG), "extractPushNotificationData - all extras: ${extras.keySet().map { "$it=${extras.get(it)}" }}")

    // Filter out system/internal keys, keep user data
    val data = JSObject()
    for (key in extras.keySet()) {
      // Skip Android/Firebase internal keys
      if (key.startsWith("android.") || key.startsWith("google.") ||
          key.startsWith("gcm.") || key == "from" || key == "collapse_key") continue
      extras.getString(key)?.let { data.put(key, it) }
    }
    Logger.debug(Logger.tags(TAG), "extractPushNotificationData - filtered data length: ${data.length()}")
    return if (data.length() > 0) data else null
  }

  private fun triggerNotificationClicked(id: Int, data: JSObject?) {
    val clickedData = JSObject()
    clickedData.put("id", id)
    if (data != null) {
      clickedData.put("data", data)
    }

    Logger.debug(Logger.tags(TAG), "triggerNotificationClicked - id: $id, hasClickedListener: $hasClickedListener, data: $data")

    if (hasClickedListener) {
      trigger("notificationClicked", clickedData)
    } else {
      Logger.debug(Logger.tags(TAG), "No click listener, storing as pending")
      pendingNotificationClick = clickedData
    }
  }

  @Command
  fun show(invoke: Invoke) {
    val notification = invoke.parseArgs(Notification::class.java)
    notification.sourceJson = invoke.getRawArgs()

    val id = manager.schedule(notification)
    if (notification.schedule != null) {
      notificationStorage.appendNotifications(listOf(notification))
    }

    invoke.resolveObject(id)
  }

  @Command
  fun batch(invoke: Invoke) {
    val args = invoke.parseArgs(BatchArgs::class.java)
    val mapper = jsonMapper()
    for (notification in args.notifications) {
      notification.sourceJson = mapper.writeValueAsString(notification)
    }

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
  fun cancelAll(invoke: Invoke) {
    val ids = notificationStorage.getSavedNotificationIds().mapNotNull { it.toIntOrNull() }
    manager.cancel(ids)
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
    val notifications = notificationStorage.getSavedNotifications()
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
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      val result = Notification.buildNotificationActiveList(notificationManager.activeNotifications)
      invoke.resolveObject(result)
    } else {
      invoke.resolveObject(emptyList<ActiveNotificationInfo>())
    }
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

  @Command
  fun unregisterForPushNotifications(invoke: Invoke) {
    if (!BuildConfig.ENABLE_PUSH_NOTIFICATIONS) {
      invoke.reject("Push notifications are disabled in this build")
      return
    }

    try {
      val firebaseMessaging = Class.forName("com.google.firebase.messaging.FirebaseMessaging")
      val getInstance = firebaseMessaging.getMethod("getInstance")
      val instance = getInstance.invoke(null)
      val deleteToken = instance.javaClass.getMethod("deleteToken")
      val task = deleteToken.invoke(instance) as com.google.android.gms.tasks.Task<*>

      task.addOnCompleteListener { completedTask ->
        if (!completedTask.isSuccessful) {
          val errorMessage = "Failed to delete FCM token: ${completedTask.exception?.message}"
          invoke.reject(errorMessage)
          return@addOnCompleteListener
        }

        // Clear cached token
        cachedToken = null
        invoke.resolve()
      }
    } catch (e: Exception) {
      val actualException = (e as? java.lang.reflect.InvocationTargetException)?.targetException ?: e
      val errorMessage = "Firebase not available: ${actualException.message ?: actualException.toString()}"
      invoke.reject(errorMessage)
    }
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
      val actualException = (e as? java.lang.reflect.InvocationTargetException)?.targetException ?: e
      val errorMessage = "Firebase not available: ${actualException.message ?: actualException.toString()}"
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

  @Command
  fun setClickListenerActive(invoke: Invoke) {
    val args = invoke.parseArgs(SetClickListenerActiveArgs::class.java)
    hasClickedListener = args.active

    // If listener just became active and we have pending click, trigger it
    if (args.active && pendingNotificationClick != null) {
      trigger("notificationClicked", pendingNotificationClick!!)
      pendingNotificationClick = null
    }

    invoke.resolve()
  }
}
