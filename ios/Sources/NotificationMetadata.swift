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

  /// Encoded as a property-list dictionary. Releases without metadata support
  /// copy only string values out of `userInfo`, so they ignore it.
  func encoded() throws -> [String: Any] {
    let data = try JSONEncoder().encode(self)
    guard let dictionary = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
      throw NotificationError.metadataEncoding("not a dictionary")
    }
    return dictionary
  }

  static func from(_ userInfo: [AnyHashable: Any]) -> NotificationMetadata? {
    guard let dictionary = userInfo[notificationMetadataKey] as? [String: Any],
      let data = try? JSONSerialization.data(withJSONObject: dictionary)
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
  /// Used for requests scheduled by versions that stored no metadata, so it
  /// only needs to cover the triggers this plugin creates. A calendar trigger
  /// maps back to the `.interval` it came from. A repeating time-interval
  /// trigger maps back to `.every` when its period fits one (see
  /// `every(seconds:)`), otherwise to a repeating `.at` with the next fire
  /// date. A one-shot time-interval trigger maps to `.at`. Anything else,
  /// such as a location trigger, is nil.
  static func from(trigger: UNNotificationTrigger?) -> NotificationSchedule? {
    switch trigger {
    case let calendar as UNCalendarNotificationTrigger:
      let components = calendar.dateComponents
      // The Rust side holds each component in a u8. A trigger outside that
      // range was not created by this plugin.
      let fields = [
        components.year, components.month, components.day, components.weekday,
        components.hour, components.minute, components.second,
      ]
      guard fields.allSatisfy({ $0.map { (0...255).contains($0) } ?? true }) else {
        return nil
      }
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
      if interval.repeats, let every = every(seconds: interval.timeInterval) {
        return every
      }
      guard let next = interval.nextTriggerDate() else {
        return nil
      }
      return .at(date: scheduleDateFormatter.string(from: next), repeating: interval.repeats)
    default:
      return nil
    }
  }

  /// The `.every` schedule with the given period, using the largest unit that
  /// divides it evenly with a count that fits the `u8` on the Rust side.
  ///
  /// Month and year periods are computed from the calendar when scheduled. A
  /// month comes back as the matching number of days. A year has no unit with
  /// a count that fits, so the caller falls back to a repeating `.at`. That
  /// fallback is lossy: `.at(date, repeating: true)` means "repeat every
  /// (date - now)", which shrinks on every read.
  static func every(seconds: TimeInterval) -> NotificationSchedule? {
    guard let total = Int(exactly: seconds), total > 0 else {
      return nil
    }
    let units: [(ScheduleEveryKind, Int)] = [
      (.twoWeeks, 1_209_600), (.week, 604_800), (.day, 86_400),
      (.hour, 3_600), (.minute, 60), (.second, 1),
    ]
    for (kind, unit) in units where total % unit == 0 {
      let count = total / unit
      guard count <= 255 else {
        return nil
      }
      return .every(interval: kind, count: count)
    }
    return nil
  }
}
