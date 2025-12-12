// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Tauri
import UserNotifications

public class NotificationHandler: NSObject, NotificationHandlerProtocol {

  public weak var plugin: Plugin?

  private var notificationsMap = [String: Notification]()
  private var hasClickedListener = false
  private var pendingNotificationClick: NotificationClickedData? = nil

  internal func saveNotification(_ key: String, _ notification: Notification) {
    notificationsMap.updateValue(notification, forKey: key)
  }

  func setClickListenerActive(_ active: Bool) {
    hasClickedListener = active

    if active, let pending = pendingNotificationClick {
      pendingNotificationClick = nil
      try? self.plugin?.trigger("notificationClicked", data: pending)
    }
  }

  public func requestPermissions(with completion: ((Bool, Error?) -> Void)? = nil) {
    let center = UNUserNotificationCenter.current()
    center.requestAuthorization(options: [.badge, .alert, .sound]) { (granted, error) in
      completion?(granted, error)
    }
  }

  public func checkPermissions(with completion: ((UNAuthorizationStatus) -> Void)? = nil) {
    let center = UNUserNotificationCenter.current()
    center.getNotificationSettings { settings in
      completion?(settings.authorizationStatus)
    }
  }

  public func willPresent(notification: UNNotification) -> UNNotificationPresentationOptions {
    // Trigger notification event for both local and push notifications
    if let notificationData = toActiveNotification(notification.request) {
      try? self.plugin?.trigger("notification", data: notificationData)
    } else {
      let notificationData = toReceivedNotification(notification.request)
      try? self.plugin?.trigger("notification", data: notificationData)
    }

    // For push notifications in foreground, don't show system notification
    // (only trigger event so developer can handle it)
    let isPushNotification = notification.request.trigger?.isKind(of: UNPushNotificationTrigger.self) == true
    if isPushNotification {
      return UNNotificationPresentationOptions.init(rawValue: 0)
    }

    // For local notifications, check if silent
    if let options = notificationsMap[notification.request.identifier] {
      if options.silent ?? false {
        return UNNotificationPresentationOptions.init(rawValue: 0)
      }
    }

    return [
      .badge,
      .sound,
      .alert,
    ]
  }

  /// Convert notification request to ReceivedNotification (for push notifications not in map)
  private func toReceivedNotification(_ request: UNNotificationRequest) -> ReceivedNotificationData {
    let content = request.content
    var extra: [String: String]? = nil

    if !content.userInfo.isEmpty {
      extra = [:]
      for (key, value) in content.userInfo {
        if let keyStr = key as? String, let valStr = value as? String {
          extra?[keyStr] = valStr
        }
      }
      if extra?.isEmpty == true {
        extra = nil
      }
    }

    return ReceivedNotificationData(
      id: Int(request.identifier) ?? -1,
      title: content.title,
      body: content.body,
      extra: extra
    )
  }

  public func didReceive(response: UNNotificationResponse) {
    let originalNotificationRequest = response.notification.request
    let actionId = response.actionIdentifier

    var actionIdValue: String
    // We turn the two default actions (open/dismiss) into generic strings
    if actionId == UNNotificationDefaultActionIdentifier {
      actionIdValue = "tap"
    } else if actionId == UNNotificationDismissActionIdentifier {
      actionIdValue = "dismiss"
    } else {
      actionIdValue = actionId
    }

    var inputValue: String? = nil
    // If the type of action was for an input type, get the value
    if let inputType = response as? UNTextInputNotificationResponse {
      inputValue = inputType.userText
    }

    // Only trigger actionPerformed for local notifications (those in our map)
    if let activeNotification = toActiveNotification(originalNotificationRequest) {
      try? self.plugin?.trigger(
        "actionPerformed",
        data: ReceivedNotification(
          actionId: actionIdValue,
          inputValue: inputValue,
          notification: activeNotification
        ))
    }

    // Handle notificationClicked for both local and push notifications
    let id = Int(originalNotificationRequest.identifier) ?? -1
    let userInfo = originalNotificationRequest.content.userInfo
    var dataDict: [String: String]? = nil
    if !userInfo.isEmpty {
      dataDict = [:]
      for (key, value) in userInfo {
        if let keyStr = key as? String, let valStr = value as? String {
          dataDict?[keyStr] = valStr
        }
      }
      if dataDict?.isEmpty == true {
        dataDict = nil
      }
    }

    let clickedData = NotificationClickedData(id: id, data: dataDict)

    if hasClickedListener {
      // Listener exists, trigger directly
      try? self.plugin?.trigger("notificationClicked", data: clickedData)
    } else {
      // No listener (cold-start), store for later
      pendingNotificationClick = clickedData
    }
  }

  func toActiveNotification(_ request: UNNotificationRequest) -> ActiveNotification? {
    guard let notificationRequest = notificationsMap[request.identifier] else {
      return nil
    }
    return ActiveNotification(
      id: Int(request.identifier) ?? -1,
      title: request.content.title,
      body: request.content.body,
      sound: notificationRequest.sound ?? "",
      actionTypeId: request.content.categoryIdentifier,
      attachments: notificationRequest.attachments
    )
  }

  func toPendingNotification(_ request: UNNotificationRequest) -> PendingNotification {
    return PendingNotification(
      id: Int(request.identifier) ?? -1,
      title: request.content.title,
      body: request.content.body
    )
  }
}

struct PendingNotification: Encodable {
  let id: Int
  let title: String
  let body: String
}

struct ActiveNotification: Encodable {
  let id: Int
  let title: String
  let body: String
  let sound: String
  let actionTypeId: String
  let attachments: [NotificationAttachment]?
}

struct ReceivedNotification: Encodable {
  let actionId: String
  let inputValue: String?
  let notification: ActiveNotification
}

struct NotificationClickedData: Encodable {
  let id: Int
  let data: [String: String]?
}

struct ReceivedNotificationData: Encodable {
  let id: Int
  let title: String
  let body: String
  let extra: [String: String]?
}
