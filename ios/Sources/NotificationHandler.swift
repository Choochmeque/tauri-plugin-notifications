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
    if var notificationData = toActiveNotification(notification.request) {
      notificationData.source = "local"
      try? self.plugin?.trigger("notification", data: notificationData)
    } else {
      var notificationData = toReceivedNotification(notification.request)
      notificationData.source = "push"
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

  /// The string-valued entries of `userInfo`, which holds only the caller's `extra`.
  private func stringExtra(from userInfo: [AnyHashable: Any]) -> [String: String]? {
    guard !userInfo.isEmpty else {
      return nil
    }

    var extra: [String: String] = [:]
    for (key, value) in userInfo {
      if let keyStr = key as? String, let valStr = value as? String {
        extra[keyStr] = valStr
      }
    }

    return extra.isEmpty ? nil : extra
  }

  /// Convert notification request to ReceivedNotification (for push notifications not in map)
  private func toReceivedNotification(_ request: UNNotificationRequest) -> ReceivedNotificationData {
    let content = request.content

    return ReceivedNotificationData(
      id: Int(request.identifier) ?? -1,
      title: content.title,
      body: content.body,
      extra: stringExtra(from: content.userInfo)
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

    // Only a tap on the body is a click; actions and dismissals went through actionPerformed above.
    guard actionId == UNNotificationDefaultActionIdentifier else {
      return
    }

    let id = Int(originalNotificationRequest.identifier) ?? -1
    let clickedData = NotificationClickedData(
      id: id,
      data: stringExtra(from: originalNotificationRequest.content.userInfo)
    )

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
      attachments: notificationRequest.attachments,
      extra: stringExtra(from: request.content.userInfo)
    )
  }

  func toPendingNotification(_ request: UNNotificationRequest) -> PendingNotification? {
    guard let notification = notificationsMap[request.identifier],
          let schedule = notification.schedule else {
      return nil
    }
    return PendingNotification(
      id: Int(request.identifier) ?? -1,
      title: request.content.title,
      body: request.content.body,
      schedule: schedule
    )
  }
}

struct PendingNotification: Encodable {
  let id: Int
  let title: String
  let body: String
  let schedule: NotificationSchedule
}

struct ActiveNotification: Encodable {
  let id: Int
  let title: String
  let body: String
  let sound: String
  let actionTypeId: String
  let attachments: [NotificationAttachment]?
  var extra: [String: String]? = nil
  var source: String = "local"
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
  var source: String = "push"
}
