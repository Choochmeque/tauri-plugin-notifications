import Foundation
import UserNotifications

/// Key under which the plugin keeps its own data in `UNNotificationContent.userInfo`.
///
/// The OS persists every request it holds, so data stored on the request
/// survives app restarts without any plugin-owned storage. Everything the
/// plugin needs later and cannot read back from the content or the trigger
/// lives under this key.
let notificationMetadataKey = "__tauri_notification"

struct NotificationMetadata: Codable {
  var schedule: NotificationSchedule?
  var silent: Bool?
  var sound: String?
  var attachments: [NotificationAttachment]?

  init(_ notification: Notification) {
    schedule = notification.schedule
    silent = notification.silent
    sound = notification.sound
    attachments = notification.attachments
  }

  /// Encoded as a JSON string because `userInfo` only accepts property-list values.
  func encoded() throws -> String {
    let data = try JSONEncoder().encode(self)
    guard let string = String(data: data, encoding: .utf8) else {
      throw NotificationError.metadataEncoding("not valid UTF-8")
    }
    return string
  }

  static func from(_ userInfo: [AnyHashable: Any]) -> NotificationMetadata? {
    guard let json = userInfo[notificationMetadataKey] as? String,
      let data = json.data(using: .utf8)
    else {
      return nil
    }
    return try? JSONDecoder().decode(NotificationMetadata.self, from: data)
  }
}

/// Formats dates the way `handleScheduledNotification` parses them.
let scheduleDateFormatter: ISO8601DateFormatter = {
  let formatter = ISO8601DateFormatter()
  formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
  formatter.timeZone = TimeZone(secondsFromGMT: 0)
  return formatter
}()

extension NotificationSchedule {
  /// Rebuilds the schedule from the trigger the OS holds.
  ///
  /// Used for requests scheduled by versions that stored no metadata. A
  /// calendar trigger maps back exactly. A time-interval trigger comes back
  /// as `.at` with its next fire date, so an `.every` schedule from an older
  /// version is reported as a repeating `.at`.
  static func from(trigger: UNNotificationTrigger?) -> NotificationSchedule? {
    switch trigger {
    case let calendar as UNCalendarNotificationTrigger:
      let components = calendar.dateComponents
      return .interval(
        interval: ScheduleInterval(
          year: components.year,
          month: components.month,
          day: components.day,
          weekday: components.weekday,
          hour: components.hour,
          minute: components.minute,
          second: components.second))
    case let interval as UNTimeIntervalNotificationTrigger:
      guard let next = interval.nextTriggerDate() else {
        return nil
      }
      return .at(date: scheduleDateFormatter.string(from: next), repeating: interval.repeats)
    default:
      return nil
    }
  }
}
