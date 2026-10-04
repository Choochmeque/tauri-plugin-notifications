import Foundation
import Tauri
import UserNotifications

public class NotificationHandler: NSObject, NotificationHandlerProtocol {

  public weak var plugin: Plugin?

  private var notificationsMap = [String: Notification]()
  private var hasClickedListener = false
  private var pendingNotificationClick: NotificationClickedData? = nil
  private var hasActionListener = false
  private var pendingNotificationActions = [ReceivedNotification]()
  private let maxPendingActions = 32
  // Serializes delegate callbacks and plugin commands.
  private let stateLock = NSRecursiveLock()

  internal func saveNotification(_ key: String, _ notification: Notification) {
    stateLock.lock()
    defer { stateLock.unlock() }
    notificationsMap.updateValue(notification, forKey: key)
  }

  func setClickListenerActive(_ active: Bool) {
    stateLock.lock()
    hasClickedListener = active
    let pending = active ? pendingNotificationClick : nil
    if pending != nil { pendingNotificationClick = nil }
    stateLock.unlock()
    if let pending { try? self.plugin?.trigger("notificationClicked", data: pending) }
  }

  func setActionListenerActive(_ active: Bool) {
    stateLock.lock()
    hasActionListener = active
    let pendingActions = active ? pendingNotificationActions : []
    if active { pendingNotificationActions.removeAll() }
    stateLock.unlock()
    guard active else { return }
    for action in pendingActions {
      try? self.plugin?.trigger("actionPerformed", data: action)
    }
  }

  private func triggerActionPerformed(_ action: ReceivedNotification) {
    stateLock.lock()
    let shouldTrigger = hasActionListener
    if !shouldTrigger && pendingNotificationActions.count >= maxPendingActions {
      pendingNotificationActions.removeFirst()
    }
    if !shouldTrigger { pendingNotificationActions.append(action) }
    stateLock.unlock()
    if shouldTrigger { try? self.plugin?.trigger("actionPerformed", data: action) }
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
    stateLock.lock()
    var event: (String, Encodable)? = nil
    // Trigger notification event for both local and push notifications
    if var notificationData = toActiveNotification(notification.request) {
      notificationData.source = "local"
      event = ("notification", notificationData)
    } else {
      var notificationData = toReceivedNotification(notification.request)
      notificationData.source = "push"
      event = ("notification", notificationData)
    }

    // For push notifications in foreground, don't show system notification
    // (only trigger event so developer can handle it)
    let isPushNotification = notification.request.trigger?.isKind(of: UNPushNotificationTrigger.self) == true
    let options: UNNotificationPresentationOptions
    if isPushNotification {
      options = UNNotificationPresentationOptions(rawValue: 0)
    } else if let local = notificationsMap[notification.request.identifier], local.silent ?? false {
      options = UNNotificationPresentationOptions(rawValue: 0)
    } else {
      options = [.badge, .sound, .alert]
    }
    stateLock.unlock()
    if let event { try? self.plugin?.trigger(event.0, data: event.1) }
    return options
  }

  /// Convert notification request to ReceivedNotification (for push notifications not in map)
  private func toReceivedNotification(_ request: UNNotificationRequest) -> ReceivedNotificationData {
    let content = request.content

    return ReceivedNotificationData(
      id: Int(request.identifier) ?? -1,
      title: content.title,
      body: content.body,
      extra: notificationExtra(content.userInfo)
    )
  }

  public func didReceive(response: UNNotificationResponse) {
    stateLock.lock()
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

    let isSystemAction = actionId == UNNotificationDefaultActionIdentifier
      || actionId == UNNotificationDismissActionIdentifier

    let actionNotification = toActiveNotification(originalNotificationRequest)
      ?? toRemoteActionNotification(originalNotificationRequest)
    let action = ReceivedNotification(
      actionId: actionIdValue,
      inputValue: inputValue,
      notification: actionNotification
    )
    let shouldTriggerAction = hasActionListener
    if !shouldTriggerAction {
      if pendingNotificationActions.count >= maxPendingActions { pendingNotificationActions.removeFirst() }
      pendingNotificationActions.append(action)
    }

    if !isSystemAction {
      stateLock.unlock()
      if shouldTriggerAction { try? self.plugin?.trigger("actionPerformed", data: action) }
      return
    }

    // Handle notificationClicked for both local and push notifications
    let id = Int(originalNotificationRequest.identifier) ?? -1
    let clickedData = NotificationClickedData(
      id: id,
      data: notificationExtra(originalNotificationRequest.content.userInfo)
    )

    let shouldTriggerClick = hasClickedListener
    if shouldTriggerClick {
      // Listener exists, trigger directly after releasing the lock.
    } else {
      // No listener (cold-start), store for later
      pendingNotificationClick = clickedData
    }
    stateLock.unlock()
    if shouldTriggerAction { try? self.plugin?.trigger("actionPerformed", data: action) }
    if shouldTriggerClick { try? self.plugin?.trigger("notificationClicked", data: clickedData) }
  }

  func toActiveNotification(_ request: UNNotificationRequest) -> ActiveNotification? {
    stateLock.lock()
    defer { stateLock.unlock() }
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
      extra: notificationExtra(request.content.userInfo)
    )
  }

  func toRemoteActionNotification(_ request: UNNotificationRequest) -> ActiveNotification {
    ActiveNotification(
      id: Int(request.identifier) ?? -1,
      title: request.content.title,
      body: request.content.body,
      sound: "",
      actionTypeId: request.content.categoryIdentifier,
      attachments: nil,
      extra: notificationExtra(request.content.userInfo),
      source: "push"
    )
  }

  private func notificationExtra(_ userInfo: [AnyHashable: Any]) -> [String: String]? {
    var extra = [String: String]()
    for (key, value) in userInfo {
      guard let key = key as? String else { continue }
      guard key != "aps" else { continue }
      if let value = value as? String {
        extra[key] = value
      } else if let value = value as? NSNumber {
        extra[key] = value.stringValue
      }
    }
    return extra.isEmpty ? nil : extra
  }

  func toPendingNotification(_ request: UNNotificationRequest) -> PendingNotification? {
    stateLock.lock()
    defer { stateLock.unlock() }
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
  let extra: [String: String]?
  var source: String

  init(
    id: Int,
    title: String,
    body: String,
    sound: String,
    actionTypeId: String,
    attachments: [NotificationAttachment]?,
    extra: [String: String]? = nil,
    source: String = "local"
  ) {
    self.id = id
    self.title = title
    self.body = body
    self.sound = sound
    self.actionTypeId = actionTypeId
    self.attachments = attachments
    self.extra = extra
    self.source = source
  }
}

struct ReceivedNotification: Encodable {
  let actionId: String
  let inputValue: String?
  let notification: ActiveNotification
}

struct NotificationClickedData: Encodable {
  let id: Int
  let data: [String: String]?

  init(id: Int, data: [String: String]?) {
    self.id = id
    self.data = data
  }
}

struct ReceivedNotificationData: Encodable {
  let id: Int
  let title: String
  let body: String
  let extra: [String: String]?
  var source: String = "push"
}
