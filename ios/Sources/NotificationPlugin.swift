// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import UserNotifications
import WebKit

enum ShowNotificationError: LocalizedError {
  case make(Error)
  case create(Error)

  var errorDescription: String? {
    switch self {
    case .make(let error):
      return "Unable to make notification: \(error)"
    case .create(let error):
      return "Unable to create notification: \(error)"
    }
  }
}

enum ScheduleEveryKind: String, Decodable {
  case year
  case month
  case twoWeeks
  case week
  case day
  case hour
  case minute
  case second
}

struct ScheduleInterval: Decodable {
  var year: Int?
  var month: Int?
  var day: Int?
  var weekday: Int?
  var hour: Int?
  var minute: Int?
  var second: Int?
}

enum NotificationSchedule: Decodable {
  case at(date: String, repeating: Bool)
  case interval(interval: ScheduleInterval)
  case every(interval: ScheduleEveryKind, count: Int)
}

struct NotificationAttachmentOptions: Codable {
  let iosUNNotificationAttachmentOptionsTypeHintKey: String?
  let iosUNNotificationAttachmentOptionsThumbnailHiddenKey: String?
  let iosUNNotificationAttachmentOptionsThumbnailClippingRectKey: String?
  let iosUNNotificationAttachmentOptionsThumbnailTimeKey: String?
}

struct NotificationAttachment: Codable {
  let id: String
  let url: String
  let options: NotificationAttachmentOptions?
}

struct Notification: Decodable {
  let id: Int
  var title: String
  var body: String?
  var extra: [String: String]?
  var schedule: NotificationSchedule?
  var attachments: [NotificationAttachment]?
  var sound: String?
  var group: String?
  var actionTypeId: String?
  var summary: String?
  var silent: Bool?
}

struct RemoveActiveNotification: Decodable {
  let id: Int
}

struct RemoveActiveArgs: Decodable {
  let notifications: [RemoveActiveNotification]
}

func showNotification(invoke: Invoke, notification: Notification)
  throws -> UNNotificationRequest
{
  var content: UNNotificationContent
  do {
    content = try makeNotificationContent(notification)
  } catch {
    throw ShowNotificationError.make(error)
  }

  var trigger: UNNotificationTrigger?

  do {
    if let schedule = notification.schedule {
      try trigger = handleScheduledNotification(schedule)
    }
  } catch {
    throw ShowNotificationError.create(error)
  }

  // Schedule the request.
  let request = UNNotificationRequest(
    identifier: "\(notification.id)", content: content, trigger: trigger
  )

  let center = UNUserNotificationCenter.current()
  center.add(request) { (error: Error?) in
    if let theError = error {
      invoke.reject(theError.localizedDescription)
    }
  }

  return request
}

struct CancelArgs: Decodable {
  let notifications: [Int]
}

struct Action: Decodable {
  let id: String
  let title: String
  var requiresAuthentication: Bool?
  var foreground: Bool?
  var destructive: Bool?
  var input: Bool?
  var inputButtonTitle: String?
  var inputPlaceholder: String?
}

struct ActionType: Decodable {
  let id: String
  let actions: [Action]
  var hiddenPreviewsBodyPlaceholder: String?
  var customDismissAction: Bool?
  var allowInCarPlay: Bool?
  var hiddenPreviewsShowTitle: Bool?
  var hiddenPreviewsShowSubtitle: Bool?
  var hiddenBodyPlaceholder: String?
}

struct RegisterActionTypesArgs: Decodable {
  let types: [ActionType]
}

struct BatchArgs: Decodable {
  let notifications: [Notification]
}

struct SetClickListenerActiveArgs: Decodable {
  let active: Bool
}

class NotificationPlugin: Plugin {
  let notificationHandler = NotificationHandler()
  let notificationManager = NotificationManager()

  #if ENABLE_PUSH_NOTIFICATIONS
  // Completion handler for push token registration
  private var pushTokenCompletion: ((Result<String, Error>) -> Void)?
  private let pushTokenTimeout: TimeInterval = 10.0
  private var pushTokenTimer: Timer?
  #endif

  override init() {
    super.init()
    notificationManager.notificationHandler = notificationHandler
    notificationHandler.plugin = self
  }

  public override func load(webview: WKWebView) {
    super.load(webview: webview)

    #if ENABLE_PUSH_NOTIFICATIONS
    // Store reference to this plugin for event triggering
    AppDelegateSwizzler.plugin = self

    // swizzle UIApplicationDelegate push methods
    AppDelegateSwizzler.swizzlePushCallbacks()
    #endif
  }

  @objc public func show(_ invoke: Invoke) throws {
    let notification = try invoke.parseArgs(Notification.self)

    let request = try showNotification(invoke: invoke, notification: notification)
    notificationHandler.saveNotification(request.identifier, notification)
    invoke.resolve(Int(request.identifier) ?? -1)
  }

  @objc public func batch(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(BatchArgs.self)
    var ids = [Int]()

    for notification in args.notifications {
      let request = try showNotification(invoke: invoke, notification: notification)
      notificationHandler.saveNotification(request.identifier, notification)
      ids.append(Int(request.identifier) ?? -1)
    }

    invoke.resolve(ids)
  }

  @objc public override func requestPermissions(_ invoke: Invoke) {
    notificationHandler.requestPermissions { granted, error in
      guard error == nil else {
        invoke.reject(error!.localizedDescription)
        return
      }

      let permissionState = granted ? "granted" : "denied"
      invoke.resolve(["permissionState": permissionState])
    }
  }

  @objc public func registerForPushNotifications(_ invoke: Invoke) {
    #if ENABLE_PUSH_NOTIFICATIONS
    // First request notification permissions
    notificationHandler.requestPermissions { [weak self] granted, error in
      guard error == nil else {
        invoke.reject(error!.localizedDescription)
        return
      }

      self?.registerForPushNotifications { result in
        switch result {
        case .success(let token):
          invoke.resolve(["deviceToken": token])
        case .failure(let error):
          invoke.reject(error.localizedDescription)
        }
      }
    }
    #else
    invoke.reject("Push notifications are disabled in this build")
    #endif
  }

  @objc public func unregisterForPushNotifications(_ invoke: Invoke) {
    #if ENABLE_PUSH_NOTIFICATIONS
    DispatchQueue.main.async {
      UIApplication.shared.unregisterForRemoteNotifications()
      invoke.resolve()
    }
    #else
    invoke.reject("Push notifications are disabled in this build")
    #endif
  }

  #if ENABLE_PUSH_NOTIFICATIONS
  private func registerForPushNotifications(completion: @escaping (Result<String, Error>) -> Void) {
    // Store completion for later
    self.pushTokenCompletion = completion

    // Set up timeout
    self.pushTokenTimer?.invalidate()
    self.pushTokenTimer = Timer.scheduledTimer(withTimeInterval: pushTokenTimeout, repeats: false) { [weak self] _ in
      self?.handlePushTokenTimeout()
    }

    // Register for remote notifications
    DispatchQueue.main.async {
      UIApplication.shared.registerForRemoteNotifications()
    }
  }

  private func handlePushTokenTimeout() {
    pushTokenTimer?.invalidate()
    pushTokenTimer = nil

    if let completion = pushTokenCompletion {
      pushTokenCompletion = nil
      let error = NSError(
        domain: "NotificationPlugin",
        code: -1,
        userInfo: [NSLocalizedDescriptionKey: "Timeout waiting for device token"]
      )
      completion(.failure(error))
    }
  }

  // Called by AppDelegateSwizzler when token is received
  func handlePushTokenReceived(_ token: String) {
    pushTokenTimer?.invalidate()
    pushTokenTimer = nil

    if let completion = pushTokenCompletion {
      pushTokenCompletion = nil
      completion(.success(token))
    }
  }

  // Called by AppDelegateSwizzler when registration fails
  func handlePushTokenError(_ error: Error) {
    pushTokenTimer?.invalidate()
    pushTokenTimer = nil

    if let completion = pushTokenCompletion {
      pushTokenCompletion = nil
      completion(.failure(error))
    }
  }
  #endif

  @objc public override func checkPermissions(_ invoke: Invoke) {
    notificationHandler.checkPermissions { status in
      let permission: String

      switch status {
      case .authorized, .ephemeral, .provisional:
        permission = "granted"
      case .denied:
        permission = "denied"
      case .notDetermined:
        permission = "prompt"
      @unknown default:
        permission = "prompt"
      }

      invoke.resolve(["permissionState": permission])
    }
  }

  @objc func cancel(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(CancelArgs.self)

    UNUserNotificationCenter.current().removePendingNotificationRequests(
      withIdentifiers: args.notifications.map { String($0) }
    )
    invoke.resolve()
  }

  @objc func getPending(_ invoke: Invoke) {
    UNUserNotificationCenter.current().getPendingNotificationRequests(completionHandler: {
      (notifications) in
      let ret = notifications.compactMap({ [weak self] (notification) -> PendingNotification? in
        return self?.notificationHandler.toPendingNotification(notification)
      })

      invoke.resolve(ret)
    })
  }

  @objc func registerActionTypes(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(RegisterActionTypesArgs.self)
    makeCategories(args.types)
    invoke.resolve()
  }

  @objc func removeActive(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(RemoveActiveArgs.self)
      UNUserNotificationCenter.current().removeDeliveredNotifications(
        withIdentifiers: args.notifications.map { String($0.id) })
      invoke.resolve()
    } catch {
      UNUserNotificationCenter.current().removeAllDeliveredNotifications()
      DispatchQueue.main.async(execute: {
        UIApplication.shared.applicationIconBadgeNumber = 0
      })
      invoke.resolve()
    }
  }

  @objc func getActive(_ invoke: Invoke) {
    UNUserNotificationCenter.current().getDeliveredNotifications(completionHandler: {
      (notifications) in
      let ret = notifications.compactMap({ (notification) -> ActiveNotification? in
        return self.notificationHandler.toActiveNotification(
          notification.request)
      })
      invoke.resolve(ret)
    })
  }

  @objc func createChannel(_ invoke: Invoke) {
    invoke.reject("not implemented")
  }

  @objc func deleteChannel(_ invoke: Invoke) {
    invoke.reject("not implemented")
  }

  @objc func listChannels(_ invoke: Invoke) {
    invoke.reject("not implemented")
  }

  @objc func setClickListenerActive(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(SetClickListenerActiveArgs.self)
      notificationHandler.setClickListenerActive(args.active)
      invoke.resolve()
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }

}

@_cdecl("init_plugin_notification")
func initPlugin() -> Plugin {
  return NotificationPlugin()
}
