import Tauri
import UserNotifications

public class NotificationHandler: NSObject, NotificationHandlerProtocol {

  public weak var plugin: Plugin?

  private var hasClickedListener = false
  private var pendingNotificationClick: NotificationClickedData? = nil

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
    let request = notification.request

    // For push notifications in foreground, don't show system notification
    // (only trigger event so developer can handle it)
    if isPushNotification(request) {
      var notificationData = toReceivedNotification(request)
      notificationData.source = "push"
      try? self.plugin?.trigger("notification", data: notificationData)
      return UNNotificationPresentationOptions.init(rawValue: 0)
    }

    var notificationData = toActiveNotification(request)
    notificationData.source = "local"
    try? self.plugin?.trigger("notification", data: notificationData)

    // For local notifications, check if silent
    if NotificationMetadata.from(request.content.userInfo)?.silent ?? false {
      return UNNotificationPresentationOptions.init(rawValue: 0)
    }

    return [
      .badge,
      .sound,
      .alert,
    ]
  }

  /// Convert notification request to ReceivedNotification (for push notifications)
  private func toReceivedNotification(_ request: UNNotificationRequest) -> ReceivedNotificationData {
    let content = request.content
    return ReceivedNotificationData(
      id: Int(request.identifier) ?? -1,
      title: content.title,
      body: content.body,
      extra: extraStrings(content.userInfo)
    )
  }

  /// The string entries of `userInfo`, without the plugin's own metadata.
  func extraStrings(_ userInfo: [AnyHashable: Any]) -> [String: String]? {
    var result = [String: String]()
    for (key, value) in userInfo {
      guard let keyStr = key as? String, keyStr != notificationMetadataKey,
        let valStr = value as? String
      else {
        continue
      }
      result[keyStr] = valStr
    }
    return result.isEmpty ? nil : result
  }

  private func isPushNotification(_ request: UNNotificationRequest) -> Bool {
    return request.trigger is UNPushNotificationTrigger
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

    // Only trigger actionPerformed for local notifications
    if !isPushNotification(originalNotificationRequest) {
      try? self.plugin?.trigger(
        "actionPerformed",
        data: ReceivedNotification(
          actionId: actionIdValue,
          inputValue: inputValue,
          notification: toActiveNotification(originalNotificationRequest)
        ))
    }

    // Handle notificationClicked for both local and push notifications
    let id = Int(originalNotificationRequest.identifier) ?? -1
    let clickedData = NotificationClickedData(
      id: id, data: extraStrings(originalNotificationRequest.content.userInfo))

    if hasClickedListener {
      // Listener exists, trigger directly
      try? self.plugin?.trigger("notificationClicked", data: clickedData)
    } else {
      // No listener (cold-start), store for later
      pendingNotificationClick = clickedData
    }
  }

  /// Builds the result from the request alone, so it works for notifications
  /// scheduled by an earlier run of the app.
  func toActiveNotification(_ request: UNNotificationRequest) -> ActiveNotification {
    let content = request.content
    let metadata = NotificationMetadata.from(content.userInfo)
    // Requests scheduled by versions without metadata still carry the
    // attachments on the content, minus the original options.
    let attachments =
      metadata?.attachments
      ?? content.attachments.map {
        NotificationAttachment(id: $0.identifier, url: $0.url.absoluteString, options: nil)
      }
    return ActiveNotification(
      id: Int(request.identifier) ?? -1,
      title: content.title,
      body: content.body,
      sound: metadata?.sound ?? "",
      actionTypeId: content.categoryIdentifier,
      attachments: attachments.isEmpty ? nil : attachments,
      source: isPushNotification(request) ? "push" : "local"
    )
  }

  /// Returns nil only for a request with neither a stored schedule nor a
  /// trigger, which the OS never reports as pending.
  func toPendingNotification(_ request: UNNotificationRequest) -> PendingNotification? {
    let metadata = NotificationMetadata.from(request.content.userInfo)
    guard let schedule = metadata?.schedule ?? NotificationSchedule.from(trigger: request.trigger)
    else {
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
