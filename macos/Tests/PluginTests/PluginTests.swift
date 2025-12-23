import XCTest
import UserNotifications
@testable import tauri_plugin_notifications

// MARK: - Test Helpers

/// Parses a JSON string into a dictionary.
private func parseJSON(_ jsonString: String) -> JsonObject? {
    guard let data = jsonString.data(using: .utf8),
          let json = try? JSONSerialization.jsonObject(with: data) as? JsonObject else {
        return nil
    }
    return json
}

/// Type alias to avoid conflicts with Foundation.Notification
typealias PluginNotification = tauri_plugin_notifications.Notification

/// Creates a minimal Notification for testing.
private func makeTestNotification(
    id: Int = 1,
    title: String = "Test Title",
    body: String? = "Test Body",
    schedule: NotificationSchedule? = nil,
    attachments: [NotificationAttachment]? = nil,
    sound: String? = nil,
    group: String? = nil,
    actionTypeId: String? = nil,
    extra: [String: String]? = nil,
    silent: Bool? = nil
) -> PluginNotification {
    return PluginNotification(
        id: id,
        title: title,
        body: body,
        extra: extra,
        schedule: schedule,
        attachments: attachments,
        sound: sound,
        group: group,
        actionTypeId: actionTypeId,
        summary: nil,
        silent: silent
    )
}

// MARK: - ScheduleEveryKind Tests

final class ScheduleEveryKindTests: XCTestCase {

    func testScheduleEveryKindRawValues() {
        XCTAssertEqual(ScheduleEveryKind.year.rawValue, "year")
        XCTAssertEqual(ScheduleEveryKind.month.rawValue, "month")
        XCTAssertEqual(ScheduleEveryKind.twoWeeks.rawValue, "twoWeeks")
        XCTAssertEqual(ScheduleEveryKind.week.rawValue, "week")
        XCTAssertEqual(ScheduleEveryKind.day.rawValue, "day")
        XCTAssertEqual(ScheduleEveryKind.hour.rawValue, "hour")
        XCTAssertEqual(ScheduleEveryKind.minute.rawValue, "minute")
        XCTAssertEqual(ScheduleEveryKind.second.rawValue, "second")
    }

    func testScheduleEveryKindFromRawValue() {
        XCTAssertEqual(ScheduleEveryKind(rawValue: "year"), .year)
        XCTAssertEqual(ScheduleEveryKind(rawValue: "month"), .month)
        XCTAssertEqual(ScheduleEveryKind(rawValue: "twoWeeks"), .twoWeeks)
        XCTAssertEqual(ScheduleEveryKind(rawValue: "week"), .week)
        XCTAssertEqual(ScheduleEveryKind(rawValue: "day"), .day)
        XCTAssertEqual(ScheduleEveryKind(rawValue: "hour"), .hour)
        XCTAssertEqual(ScheduleEveryKind(rawValue: "minute"), .minute)
        XCTAssertEqual(ScheduleEveryKind(rawValue: "second"), .second)
        XCTAssertNil(ScheduleEveryKind(rawValue: "invalid"))
    }

    func testScheduleEveryKindEncodable() throws {
        let encoder = JSONEncoder()

        let yearData = try encoder.encode(ScheduleEveryKind.year)
        XCTAssertEqual(String(data: yearData, encoding: .utf8), "\"year\"")

        let monthData = try encoder.encode(ScheduleEveryKind.month)
        XCTAssertEqual(String(data: monthData, encoding: .utf8), "\"month\"")
    }

    func testScheduleEveryKindDecodable() throws {
        let decoder = JSONDecoder()

        let yearData = "\"year\"".data(using: .utf8)!
        let year = try decoder.decode(ScheduleEveryKind.self, from: yearData)
        XCTAssertEqual(year, .year)

        let weekData = "\"week\"".data(using: .utf8)!
        let week = try decoder.decode(ScheduleEveryKind.self, from: weekData)
        XCTAssertEqual(week, .week)
    }
}

// MARK: - ScheduleInterval Tests

final class ScheduleIntervalTests: XCTestCase {

    func testScheduleIntervalDecoding() throws {
        let json = """
        {"year": 2024, "month": 12, "day": 25, "hour": 10, "minute": 30, "second": 0}
        """
        let data = json.data(using: .utf8)!
        let interval = try JSONDecoder().decode(ScheduleInterval.self, from: data)

        XCTAssertEqual(interval.year, 2024)
        XCTAssertEqual(interval.month, 12)
        XCTAssertEqual(interval.day, 25)
        XCTAssertEqual(interval.hour, 10)
        XCTAssertEqual(interval.minute, 30)
        XCTAssertEqual(interval.second, 0)
        XCTAssertNil(interval.weekday)
    }

    func testScheduleIntervalPartialDecoding() throws {
        let json = """
        {"hour": 9, "minute": 0}
        """
        let data = json.data(using: .utf8)!
        let interval = try JSONDecoder().decode(ScheduleInterval.self, from: data)

        XCTAssertNil(interval.year)
        XCTAssertNil(interval.month)
        XCTAssertNil(interval.day)
        XCTAssertEqual(interval.hour, 9)
        XCTAssertEqual(interval.minute, 0)
        XCTAssertNil(interval.second)
        XCTAssertNil(interval.weekday)
    }

    func testScheduleIntervalWithWeekday() throws {
        let json = """
        {"weekday": 2, "hour": 14}
        """
        let data = json.data(using: .utf8)!
        let interval = try JSONDecoder().decode(ScheduleInterval.self, from: data)

        XCTAssertEqual(interval.weekday, 2)
        XCTAssertEqual(interval.hour, 14)
    }
}

// MARK: - NotificationSchedule Tests

final class NotificationScheduleTests: XCTestCase {

    func testNotificationScheduleAtDecoding() throws {
        let json = """
        {"at": {"date": "2024-12-25T10:00:00.000Z", "repeating": false}}
        """
        let data = json.data(using: .utf8)!
        let schedule = try JSONDecoder().decode(NotificationSchedule.self, from: data)

        if case .at(let date, let repeating) = schedule {
            XCTAssertEqual(date, "2024-12-25T10:00:00.000Z")
            XCTAssertFalse(repeating)
        } else {
            XCTFail("Expected .at schedule")
        }
    }

    func testNotificationScheduleIntervalDecoding() throws {
        let json = """
        {"interval": {"interval": {"hour": 9, "minute": 0}}}
        """
        let data = json.data(using: .utf8)!
        let schedule = try JSONDecoder().decode(NotificationSchedule.self, from: data)

        if case .interval(let interval) = schedule {
            XCTAssertEqual(interval.hour, 9)
            XCTAssertEqual(interval.minute, 0)
        } else {
            XCTFail("Expected .interval schedule")
        }
    }

    func testNotificationScheduleEveryDecoding() throws {
        let json = """
        {"every": {"interval": "hour", "count": 2}}
        """
        let data = json.data(using: .utf8)!
        let schedule = try JSONDecoder().decode(NotificationSchedule.self, from: data)

        if case .every(let interval, let count) = schedule {
            XCTAssertEqual(interval, .hour)
            XCTAssertEqual(count, 2)
        } else {
            XCTFail("Expected .every schedule")
        }
    }
}

// MARK: - Notification Tests

final class NotificationTests: XCTestCase {

    func testNotificationMinimalDecoding() throws {
        let json = """
        {"id": 123, "title": "Hello"}
        """
        let data = json.data(using: .utf8)!
        let notification = try JSONDecoder().decode(PluginNotification.self, from: data)

        XCTAssertEqual(notification.id, 123)
        XCTAssertEqual(notification.title, "Hello")
        XCTAssertNil(notification.body)
        XCTAssertNil(notification.extra)
        XCTAssertNil(notification.schedule)
        XCTAssertNil(notification.attachments)
        XCTAssertNil(notification.sound)
        XCTAssertNil(notification.group)
        XCTAssertNil(notification.actionTypeId)
        XCTAssertNil(notification.silent)
    }

    func testNotificationFullDecoding() throws {
        let json = """
        {
            "id": 456,
            "title": "Reminder",
            "body": "Don't forget!",
            "extra": {"key1": "value1", "key2": "value2"},
            "sound": "notification.wav",
            "group": "reminders",
            "actionTypeId": "reminder_actions",
            "summary": "A reminder",
            "silent": false
        }
        """
        let data = json.data(using: .utf8)!
        let notification = try JSONDecoder().decode(PluginNotification.self, from: data)

        XCTAssertEqual(notification.id, 456)
        XCTAssertEqual(notification.title, "Reminder")
        XCTAssertEqual(notification.body, "Don't forget!")
        XCTAssertEqual(notification.extra?["key1"], "value1")
        XCTAssertEqual(notification.extra?["key2"], "value2")
        XCTAssertEqual(notification.sound, "notification.wav")
        XCTAssertEqual(notification.group, "reminders")
        XCTAssertEqual(notification.actionTypeId, "reminder_actions")
        XCTAssertEqual(notification.summary, "A reminder")
        XCTAssertEqual(notification.silent, false)
    }

    func testNotificationWithSchedule() throws {
        let json = """
        {
            "id": 1,
            "title": "Scheduled",
            "schedule": {"every": {"interval": "day", "count": 1}}
        }
        """
        let data = json.data(using: .utf8)!
        let notification = try JSONDecoder().decode(PluginNotification.self, from: data)

        XCTAssertNotNil(notification.schedule)
        if case .every(let interval, let count) = notification.schedule! {
            XCTAssertEqual(interval, .day)
            XCTAssertEqual(count, 1)
        } else {
            XCTFail("Expected .every schedule")
        }
    }

    func testNotificationWithAttachments() throws {
        let json = """
        {
            "id": 1,
            "title": "With Image",
            "attachments": [
                {"id": "img1", "url": "file:///path/to/image.png"}
            ]
        }
        """
        let data = json.data(using: .utf8)!
        let notification = try JSONDecoder().decode(PluginNotification.self, from: data)

        XCTAssertEqual(notification.attachments?.count, 1)
        XCTAssertEqual(notification.attachments?[0].id, "img1")
        XCTAssertEqual(notification.attachments?[0].url, "file:///path/to/image.png")
    }
}

// MARK: - Action Tests

final class ActionTests: XCTestCase {

    func testActionMinimalDecoding() throws {
        let json = """
        {"id": "action1", "title": "Accept"}
        """
        let data = json.data(using: .utf8)!
        let action = try JSONDecoder().decode(Action.self, from: data)

        XCTAssertEqual(action.id, "action1")
        XCTAssertEqual(action.title, "Accept")
        XCTAssertNil(action.requiresAuthentication)
        XCTAssertNil(action.foreground)
        XCTAssertNil(action.destructive)
        XCTAssertNil(action.input)
    }

    func testActionFullDecoding() throws {
        let json = """
        {
            "id": "reply",
            "title": "Reply",
            "requiresAuthentication": true,
            "foreground": true,
            "destructive": false,
            "input": true,
            "inputButtonTitle": "Send",
            "inputPlaceholder": "Type your reply..."
        }
        """
        let data = json.data(using: .utf8)!
        let action = try JSONDecoder().decode(Action.self, from: data)

        XCTAssertEqual(action.id, "reply")
        XCTAssertEqual(action.title, "Reply")
        XCTAssertEqual(action.requiresAuthentication, true)
        XCTAssertEqual(action.foreground, true)
        XCTAssertEqual(action.destructive, false)
        XCTAssertEqual(action.input, true)
        XCTAssertEqual(action.inputButtonTitle, "Send")
        XCTAssertEqual(action.inputPlaceholder, "Type your reply...")
    }

    func testDestructiveAction() throws {
        let json = """
        {"id": "delete", "title": "Delete", "destructive": true}
        """
        let data = json.data(using: .utf8)!
        let action = try JSONDecoder().decode(Action.self, from: data)

        XCTAssertEqual(action.destructive, true)
    }
}

// MARK: - ActionType Tests

final class ActionTypeTests: XCTestCase {

    func testActionTypeDecoding() throws {
        let json = """
        {
            "id": "message_actions",
            "actions": [
                {"id": "reply", "title": "Reply"},
                {"id": "delete", "title": "Delete", "destructive": true}
            ]
        }
        """
        let data = json.data(using: .utf8)!
        let actionType = try JSONDecoder().decode(ActionType.self, from: data)

        XCTAssertEqual(actionType.id, "message_actions")
        XCTAssertEqual(actionType.actions.count, 2)
        XCTAssertEqual(actionType.actions[0].id, "reply")
        XCTAssertEqual(actionType.actions[1].destructive, true)
    }

    func testActionTypeWithOptions() throws {
        let json = """
        {
            "id": "secure_actions",
            "actions": [{"id": "confirm", "title": "Confirm"}],
            "customDismissAction": true,
            "hiddenPreviewsShowTitle": true
        }
        """
        let data = json.data(using: .utf8)!
        let actionType = try JSONDecoder().decode(ActionType.self, from: data)

        XCTAssertEqual(actionType.customDismissAction, true)
        XCTAssertEqual(actionType.hiddenPreviewsShowTitle, true)
    }
}

// MARK: - CancelArgs Tests

final class CancelArgsTests: XCTestCase {

    func testCancelArgsDecoding() throws {
        let json = """
        {"notifications": [1, 2, 3, 4, 5]}
        """
        let data = json.data(using: .utf8)!
        let args = try JSONDecoder().decode(CancelArgs.self, from: data)

        XCTAssertEqual(args.notifications, [1, 2, 3, 4, 5])
    }

    func testCancelArgsEmptyArray() throws {
        let json = """
        {"notifications": []}
        """
        let data = json.data(using: .utf8)!
        let args = try JSONDecoder().decode(CancelArgs.self, from: data)

        XCTAssertTrue(args.notifications.isEmpty)
    }
}

// MARK: - RemoveActiveArgs Tests

final class RemoveActiveArgsTests: XCTestCase {

    func testRemoveActiveArgsDecoding() throws {
        let json = """
        {"notifications": [{"id": 1}, {"id": 2}, {"id": 3}]}
        """
        let data = json.data(using: .utf8)!
        let args = try JSONDecoder().decode(RemoveActiveArgs.self, from: data)

        XCTAssertEqual(args.notifications.count, 3)
        XCTAssertEqual(args.notifications[0].id, 1)
        XCTAssertEqual(args.notifications[1].id, 2)
        XCTAssertEqual(args.notifications[2].id, 3)
    }
}

// MARK: - SetClickListenerActiveArgs Tests

final class SetClickListenerActiveArgsTests: XCTestCase {

    func testSetClickListenerActiveArgsTrue() throws {
        let json = """
        {"active": true}
        """
        let data = json.data(using: .utf8)!
        let args = try JSONDecoder().decode(SetClickListenerActiveArgs.self, from: data)

        XCTAssertTrue(args.active)
    }

    func testSetClickListenerActiveArgsFalse() throws {
        let json = """
        {"active": false}
        """
        let data = json.data(using: .utf8)!
        let args = try JSONDecoder().decode(SetClickListenerActiveArgs.self, from: data)

        XCTAssertFalse(args.active)
    }
}

// MARK: - Helper Function Tests

final class HelperFunctionTests: XCTestCase {

    // MARK: - getDateComponents Tests

    func testGetDateComponentsFull() {
        let interval = ScheduleInterval(
            year: 2024,
            month: 12,
            day: 25,
            weekday: nil,
            hour: 10,
            minute: 30,
            second: 0
        )

        let components = getDateComponents(interval)

        XCTAssertEqual(components.year, 2024)
        XCTAssertEqual(components.month, 12)
        XCTAssertEqual(components.day, 25)
        XCTAssertEqual(components.hour, 10)
        XCTAssertEqual(components.minute, 30)
        XCTAssertEqual(components.second, 0)
        XCTAssertNil(components.weekday)
    }

    func testGetDateComponentsPartial() {
        let interval = ScheduleInterval(
            year: nil,
            month: nil,
            day: nil,
            weekday: 2,
            hour: 9,
            minute: 0,
            second: nil
        )

        let components = getDateComponents(interval)

        XCTAssertNil(components.year)
        XCTAssertNil(components.month)
        XCTAssertNil(components.day)
        XCTAssertEqual(components.weekday, 2)
        XCTAssertEqual(components.hour, 9)
        XCTAssertEqual(components.minute, 0)
        XCTAssertNil(components.second)
    }

    // MARK: - getRepeatDateInterval Tests

    func testGetRepeatDateIntervalYear() {
        let interval = getRepeatDateInterval(.year, 1)

        XCTAssertNotNil(interval)
        // A year should be roughly 365 days
        let days = interval!.duration / (24 * 60 * 60)
        XCTAssertTrue(days >= 365 && days <= 366)
    }

    func testGetRepeatDateIntervalMonth() {
        let interval = getRepeatDateInterval(.month, 1)

        XCTAssertNotNil(interval)
        // A month is roughly 28-31 days
        let days = interval!.duration / (24 * 60 * 60)
        XCTAssertTrue(days >= 28 && days <= 31)
    }

    func testGetRepeatDateIntervalTwoWeeks() {
        let interval = getRepeatDateInterval(.twoWeeks, 1)

        XCTAssertNotNil(interval)
        let days = interval!.duration / (24 * 60 * 60)
        XCTAssertEqual(days, 14, accuracy: 0.01)
    }

    func testGetRepeatDateIntervalWeek() {
        let interval = getRepeatDateInterval(.week, 1)

        XCTAssertNotNil(interval)
        let days = interval!.duration / (24 * 60 * 60)
        XCTAssertEqual(days, 7, accuracy: 0.01)
    }

    func testGetRepeatDateIntervalDay() {
        let interval = getRepeatDateInterval(.day, 1)

        XCTAssertNotNil(interval)
        let hours = interval!.duration / (60 * 60)
        XCTAssertEqual(hours, 24, accuracy: 0.01)
    }

    func testGetRepeatDateIntervalHour() {
        let interval = getRepeatDateInterval(.hour, 1)

        XCTAssertNotNil(interval)
        let minutes = interval!.duration / 60
        XCTAssertEqual(minutes, 60, accuracy: 0.01)
    }

    func testGetRepeatDateIntervalMinute() {
        let interval = getRepeatDateInterval(.minute, 1)

        XCTAssertNotNil(interval)
        XCTAssertEqual(interval!.duration, 60, accuracy: 0.01)
    }

    func testGetRepeatDateIntervalSecond() {
        let interval = getRepeatDateInterval(.second, 30)

        XCTAssertNotNil(interval)
        XCTAssertEqual(interval!.duration, 30, accuracy: 0.01)
    }

    func testGetRepeatDateIntervalWithCount() {
        let interval = getRepeatDateInterval(.hour, 3)

        XCTAssertNotNil(interval)
        let hours = interval!.duration / (60 * 60)
        XCTAssertEqual(hours, 3, accuracy: 0.01)
    }

    // MARK: - makeActionOptions Tests

    func testMakeActionOptionsForeground() {
        let action = Action(
            id: "test",
            title: "Test",
            requiresAuthentication: nil,
            foreground: true,
            destructive: nil,
            input: nil,
            inputButtonTitle: nil,
            inputPlaceholder: nil
        )

        let options = makeActionOptions(action)
        XCTAssertEqual(options, .foreground)
    }

    func testMakeActionOptionsDestructive() {
        let action = Action(
            id: "test",
            title: "Test",
            requiresAuthentication: nil,
            foreground: nil,
            destructive: true,
            input: nil,
            inputButtonTitle: nil,
            inputPlaceholder: nil
        )

        let options = makeActionOptions(action)
        XCTAssertEqual(options, .destructive)
    }

    func testMakeActionOptionsAuthRequired() {
        let action = Action(
            id: "test",
            title: "Test",
            requiresAuthentication: true,
            foreground: nil,
            destructive: nil,
            input: nil,
            inputButtonTitle: nil,
            inputPlaceholder: nil
        )

        let options = makeActionOptions(action)
        XCTAssertEqual(options, .authenticationRequired)
    }

    func testMakeActionOptionsNone() {
        let action = Action(
            id: "test",
            title: "Test",
            requiresAuthentication: nil,
            foreground: nil,
            destructive: nil,
            input: nil,
            inputButtonTitle: nil,
            inputPlaceholder: nil
        )

        let options = makeActionOptions(action)
        XCTAssertEqual(options.rawValue, 0)
    }

    // MARK: - makeCategoryOptions Tests

    func testMakeCategoryOptionsCustomDismiss() {
        let actionType = ActionType(
            id: "test",
            actions: [],
            hiddenPreviewsBodyPlaceholder: nil,
            customDismissAction: true,
            allowInCarPlay: nil,
            hiddenPreviewsShowTitle: nil,
            hiddenPreviewsShowSubtitle: nil,
            hiddenBodyPlaceholder: nil
        )

        let options = makeCategoryOptions(actionType)
        XCTAssertEqual(options, .customDismissAction)
    }

    func testMakeCategoryOptionsShowTitle() {
        let actionType = ActionType(
            id: "test",
            actions: [],
            hiddenPreviewsBodyPlaceholder: nil,
            customDismissAction: nil,
            allowInCarPlay: nil,
            hiddenPreviewsShowTitle: true,
            hiddenPreviewsShowSubtitle: nil,
            hiddenBodyPlaceholder: nil
        )

        let options = makeCategoryOptions(actionType)
        XCTAssertEqual(options, .hiddenPreviewsShowTitle)
    }

    func testMakeCategoryOptionsShowSubtitle() {
        let actionType = ActionType(
            id: "test",
            actions: [],
            hiddenPreviewsBodyPlaceholder: nil,
            customDismissAction: nil,
            allowInCarPlay: nil,
            hiddenPreviewsShowTitle: nil,
            hiddenPreviewsShowSubtitle: true,
            hiddenBodyPlaceholder: nil
        )

        let options = makeCategoryOptions(actionType)
        XCTAssertEqual(options, .hiddenPreviewsShowSubtitle)
    }

    func testMakeCategoryOptionsNone() {
        let actionType = ActionType(
            id: "test",
            actions: [],
            hiddenPreviewsBodyPlaceholder: nil,
            customDismissAction: nil,
            allowInCarPlay: nil,
            hiddenPreviewsShowTitle: nil,
            hiddenPreviewsShowSubtitle: nil,
            hiddenBodyPlaceholder: nil
        )

        let options = makeCategoryOptions(actionType)
        XCTAssertEqual(options.rawValue, 0)
    }

    // MARK: - makeActions Tests

    func testMakeActionsBasic() {
        let actions = [
            Action(
                id: "accept",
                title: "Accept",
                requiresAuthentication: nil,
                foreground: true,
                destructive: nil,
                input: nil,
                inputButtonTitle: nil,
                inputPlaceholder: nil
            ),
            Action(
                id: "decline",
                title: "Decline",
                requiresAuthentication: nil,
                foreground: nil,
                destructive: true,
                input: nil,
                inputButtonTitle: nil,
                inputPlaceholder: nil
            )
        ]

        let unActions = makeActions(actions)

        XCTAssertEqual(unActions.count, 2)
        XCTAssertEqual(unActions[0].identifier, "accept")
        XCTAssertEqual(unActions[0].title, "Accept")
        XCTAssertEqual(unActions[1].identifier, "decline")
        XCTAssertEqual(unActions[1].title, "Decline")
    }

    func testMakeActionsWithInput() {
        let actions = [
            Action(
                id: "reply",
                title: "Reply",
                requiresAuthentication: nil,
                foreground: nil,
                destructive: nil,
                input: true,
                inputButtonTitle: "Send",
                inputPlaceholder: "Type here..."
            )
        ]

        let unActions = makeActions(actions)

        XCTAssertEqual(unActions.count, 1)
        XCTAssertTrue(unActions[0] is UNTextInputNotificationAction)

        let textAction = unActions[0] as! UNTextInputNotificationAction
        XCTAssertEqual(textAction.identifier, "reply")
        XCTAssertEqual(textAction.textInputButtonTitle, "Send")
        XCTAssertEqual(textAction.textInputPlaceholder, "Type here...")
    }

    func testMakeActionsEmptyArray() {
        let actions: [Action] = []
        let unActions = makeActions(actions)
        XCTAssertTrue(unActions.isEmpty)
    }

    // MARK: - makeAttachmentUrl Tests

    func testMakeAttachmentUrlValid() {
        let url = makeAttachmentUrl("file:///path/to/image.png")
        XCTAssertNotNil(url)
        XCTAssertEqual(url?.scheme, "file")
    }

    func testMakeAttachmentUrlHttp() {
        let url = makeAttachmentUrl("https://example.com/image.png")
        XCTAssertNotNil(url)
        XCTAssertEqual(url?.scheme, "https")
    }

    // MARK: - makeAttachmentOptions Tests

    func testMakeAttachmentOptionsWithTypeHint() {
        let options = NotificationAttachmentOptions(
            iosUNNotificationAttachmentOptionsTypeHintKey: "public.png",
            iosUNNotificationAttachmentOptionsThumbnailHiddenKey: nil,
            iosUNNotificationAttachmentOptionsThumbnailClippingRectKey: nil,
            iosUNNotificationAttachmentOptionsThumbnailTimeKey: nil
        )

        let result = makeAttachmentOptions(options)
        XCTAssertEqual(result[UNNotificationAttachmentOptionsTypeHintKey] as? String, "public.png")
    }

    func testMakeAttachmentOptionsEmpty() {
        let options = NotificationAttachmentOptions(
            iosUNNotificationAttachmentOptionsTypeHintKey: nil,
            iosUNNotificationAttachmentOptionsThumbnailHiddenKey: nil,
            iosUNNotificationAttachmentOptionsThumbnailClippingRectKey: nil,
            iosUNNotificationAttachmentOptionsThumbnailTimeKey: nil
        )

        let result = makeAttachmentOptions(options)
        XCTAssertTrue(result.isEmpty)
    }
}

// MARK: - NotificationContent Tests

final class NotificationContentTests: XCTestCase {

    func testMakeNotificationContentBasic() throws {
        let notification = makeTestNotification(title: "Test Title", body: "Test Body")

        let content = try makeNotificationContent(notification)

        XCTAssertEqual(content.title, "Test Title")
        XCTAssertEqual(content.body, "Test Body")
    }

    func testMakeNotificationContentWithExtra() throws {
        let notification = makeTestNotification(
            extra: ["key1": "value1", "key2": "value2"]
        )

        let content = try makeNotificationContent(notification)

        let extra = content.userInfo["__EXTRA__"] as? [String: String]
        XCTAssertNotNil(extra)
        XCTAssertEqual(extra?["key1"], "value1")
        XCTAssertEqual(extra?["key2"], "value2")
    }

    func testMakeNotificationContentWithActionType() throws {
        let notification = makeTestNotification(actionTypeId: "message_actions")

        let content = try makeNotificationContent(notification)

        XCTAssertEqual(content.categoryIdentifier, "message_actions")
    }

    func testMakeNotificationContentWithGroup() throws {
        let notification = makeTestNotification(group: "messages")

        let content = try makeNotificationContent(notification)

        XCTAssertEqual(content.threadIdentifier, "messages")
    }

    func testMakeNotificationContentWithSound() throws {
        let notification = makeTestNotification(sound: "custom_sound.wav")

        let content = try makeNotificationContent(notification)

        XCTAssertNotNil(content.sound)
    }

    func testMakeNotificationContentWithoutBody() throws {
        let notification = makeTestNotification(body: nil)

        let content = try makeNotificationContent(notification)

        XCTAssertEqual(content.title, "Test Title")
        XCTAssertEqual(content.body, "")
    }
}

// MARK: - handleScheduledNotification Tests

final class ScheduleHandlerTests: XCTestCase {

    func testHandleScheduledNotificationEveryHour() throws {
        let schedule = NotificationSchedule.every(interval: .hour, count: 2)

        let trigger = try handleScheduledNotification(schedule)

        XCTAssertNotNil(trigger)
        XCTAssertTrue(trigger is UNTimeIntervalNotificationTrigger)

        let timeTrigger = trigger as! UNTimeIntervalNotificationTrigger
        XCTAssertTrue(timeTrigger.repeats)
        // 2 hours = 7200 seconds
        XCTAssertEqual(timeTrigger.timeInterval, 7200, accuracy: 1)
    }

    func testHandleScheduledNotificationEveryDay() throws {
        let schedule = NotificationSchedule.every(interval: .day, count: 1)

        let trigger = try handleScheduledNotification(schedule)

        XCTAssertNotNil(trigger)
        XCTAssertTrue(trigger is UNTimeIntervalNotificationTrigger)

        let timeTrigger = trigger as! UNTimeIntervalNotificationTrigger
        XCTAssertTrue(timeTrigger.repeats)
        // 1 day = 86400 seconds
        XCTAssertEqual(timeTrigger.timeInterval, 86400, accuracy: 1)
    }

    func testHandleScheduledNotificationInterval() throws {
        let interval = ScheduleInterval(
            year: nil,
            month: nil,
            day: nil,
            weekday: nil,
            hour: 9,
            minute: 30,
            second: nil
        )
        let schedule = NotificationSchedule.interval(interval: interval)

        let trigger = try handleScheduledNotification(schedule)

        XCTAssertNotNil(trigger)
        XCTAssertTrue(trigger is UNCalendarNotificationTrigger)

        let calendarTrigger = trigger as! UNCalendarNotificationTrigger
        XCTAssertTrue(calendarTrigger.repeats)
        XCTAssertEqual(calendarTrigger.dateComponents.hour, 9)
        XCTAssertEqual(calendarTrigger.dateComponents.minute, 30)
    }

    func testHandleScheduledNotificationTooShortInterval() throws {
        // 30 seconds is less than 60 seconds minimum
        let schedule = NotificationSchedule.every(interval: .second, count: 30)

        XCTAssertThrowsError(try handleScheduledNotification(schedule)) { error in
            XCTAssertTrue(error is NotificationError)
            if case NotificationError.triggerRepeatIntervalTooShort = error {
                // Expected
            } else {
                XCTFail("Expected triggerRepeatIntervalTooShort error")
            }
        }
    }

    func testHandleScheduledNotificationAtFutureDate() throws {
        let futureDate = Calendar.current.date(byAdding: .hour, value: 1, to: Date())!
        let dateFormatter = DateFormatter()
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.timeZone = TimeZone(secondsFromGMT: 0)
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"
        let dateString = dateFormatter.string(from: futureDate)

        let schedule = NotificationSchedule.at(date: dateString, repeating: false)

        let trigger = try handleScheduledNotification(schedule)

        XCTAssertNotNil(trigger)
        XCTAssertTrue(trigger is UNTimeIntervalNotificationTrigger)

        let timeTrigger = trigger as! UNTimeIntervalNotificationTrigger
        XCTAssertFalse(timeTrigger.repeats)
        // Should be roughly 3600 seconds (1 hour)
        XCTAssertTrue(timeTrigger.timeInterval > 3500 && timeTrigger.timeInterval < 3700)
    }

    func testHandleScheduledNotificationAtPastDate() throws {
        let pastDate = Calendar.current.date(byAdding: .hour, value: -1, to: Date())!
        let dateFormatter = DateFormatter()
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.timeZone = TimeZone(secondsFromGMT: 0)
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"
        let dateString = dateFormatter.string(from: pastDate)

        let schedule = NotificationSchedule.at(date: dateString, repeating: false)

        XCTAssertThrowsError(try handleScheduledNotification(schedule)) { error in
            XCTAssertTrue(error is NotificationError)
            if case NotificationError.pastScheduledTime = error {
                // Expected
            } else {
                XCTFail("Expected pastScheduledTime error")
            }
        }
    }

    func testHandleScheduledNotificationInvalidDate() throws {
        let schedule = NotificationSchedule.at(date: "invalid-date", repeating: false)

        XCTAssertThrowsError(try handleScheduledNotification(schedule)) { error in
            XCTAssertTrue(error is NotificationError)
            if case NotificationError.invalidDate(_) = error {
                // Expected
            } else {
                XCTFail("Expected invalidDate error")
            }
        }
    }

    func testHandleScheduledNotificationRepeatingTooShort() throws {
        // Create a date just 30 seconds in the future with repeating
        let futureDate = Calendar.current.date(byAdding: .second, value: 30, to: Date())!
        let dateFormatter = DateFormatter()
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.timeZone = TimeZone(secondsFromGMT: 0)
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"
        let dateString = dateFormatter.string(from: futureDate)

        let schedule = NotificationSchedule.at(date: dateString, repeating: true)

        XCTAssertThrowsError(try handleScheduledNotification(schedule)) { error in
            XCTAssertTrue(error is NotificationError)
            if case NotificationError.triggerRepeatIntervalTooShort = error {
                // Expected
            } else {
                XCTFail("Expected triggerRepeatIntervalTooShort error")
            }
        }
    }
}

// MARK: - NotificationError Tests

final class NotificationErrorTests: XCTestCase {

    func testErrorDescriptions() {
        XCTAssertEqual(
            NotificationError.triggerRepeatIntervalTooShort.errorDescription,
            "Schedule interval too short, must be a least 1 minute"
        )

        XCTAssertEqual(
            NotificationError.attachmentFileNotFound(path: "/path/to/file").errorDescription,
            "Unable to find file /path/to/file for attachment"
        )

        XCTAssertEqual(
            NotificationError.attachmentUnableToCreate("some error").errorDescription,
            "Failed to create attachment: some error"
        )

        XCTAssertEqual(
            NotificationError.pastScheduledTime.errorDescription,
            "Scheduled time must be *after* current time"
        )

        XCTAssertEqual(
            NotificationError.invalidDate("bad-date").errorDescription,
            "Could not parse date bad-date"
        )
    }
}

// MARK: - Plugin Initialization Tests
// Note: Some plugin tests require a full application environment with notification center access.
// Tests that crash in pure unit test environments are skipped.

final class PluginTests: XCTestCase {
    var plugin: NotificationPlugin?

    override func setUp() {
        super.setUp()
        // Plugin initialization may fail in pure unit test environment
        // We'll handle this gracefully
    }

    override func tearDown() {
        plugin = nil
        super.tearDown()
    }

    private func getPlugin() throws -> NotificationPlugin {
        if plugin == nil {
            plugin = initPlugin()
        }
        return try XCTUnwrap(plugin)
    }

    func testPluginInitialization() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Plugin initialization requires a full application environment")
    }

    func testCheckPermissionsReturnsValidJSON() async throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testCancelWithValidArgs() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testCancelWithEmptyArray() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testCancelAllDoesNotThrow() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testRegisterActionTypesWithValidArgs() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testRegisterActionTypesWithEmptyTypes() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testRemoveActiveWithValidArgs() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testRemoveAllActiveDoesNotThrow() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testSetClickListenerActiveTrue() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testSetClickListenerActiveFalse() throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testGetPendingReturnsValidJSON() async throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }

    func testGetActiveReturnsValidJSON() async throws {
        // Skip in environments where notification center isn't available
        throw XCTSkip("Requires notification center access")
    }
}

// MARK: - RustString Extension Tests

final class RustStringExtensionTests: XCTestCase {

    func testDecodeValidJSON() throws {
        let rustString = RustString("{\"notifications\": [1, 2, 3]}")
        let result = try rustString.decode(CancelArgs.self)
        XCTAssertEqual(result.notifications, [1, 2, 3])
    }

    func testDecodeInvalidJSON() {
        let rustString = RustString("invalid json")

        do {
            _ = try rustString.decode(CancelArgs.self)
            XCTFail("Should have thrown an error")
        } catch {
            // Expected
            XCTAssertTrue(error is FFIResult)
        }
    }

    func testDecodeComplexType() throws {
        let json = """
        {
            "types": [
                {
                    "id": "msg",
                    "actions": [
                        {"id": "reply", "title": "Reply", "input": true}
                    ]
                }
            ]
        }
        """
        let rustString = RustString(json)
        let result = try rustString.decode(RegisterActionTypesArgs.self)

        XCTAssertEqual(result.types.count, 1)
        XCTAssertEqual(result.types[0].id, "msg")
        XCTAssertEqual(result.types[0].actions[0].input, true)
    }
}

// MARK: - Encodable Extension Tests

final class EncodableExtensionTests: XCTestCase {

    func testActiveNotificationToJSON() throws {
        let notification = ActiveNotification(
            id: 123,
            title: "Test",
            body: "Body",
            sound: "default",
            actionTypeId: "actions",
            attachments: nil
        )

        let jsonString = try notification.toJSONString()
        let data = try XCTUnwrap(jsonString.data(using: .utf8))
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        XCTAssertEqual(json["id"] as? Int, 123)
        XCTAssertEqual(json["title"] as? String, "Test")
        XCTAssertEqual(json["body"] as? String, "Body")
        XCTAssertEqual(json["sound"] as? String, "default")
        XCTAssertEqual(json["actionTypeId"] as? String, "actions")
    }

    func testPendingNotificationToJSON() throws {
        let notification = PendingNotification(
            id: 456,
            title: "Pending",
            body: "Pending body",
            schedule: .every(interval: .hour, count: 1)
        )

        let jsonString = try notification.toJSONString()
        let data = try XCTUnwrap(jsonString.data(using: .utf8))
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        XCTAssertEqual(json["id"] as? Int, 456)
        XCTAssertEqual(json["title"] as? String, "Pending")
    }

    func testNotificationClickedDataToJSON() throws {
        let clickedData = NotificationClickedData(
            id: 789,
            data: ["action": "tap", "source": "notification"]
        )

        let jsonString = try clickedData.toJSONString()
        let data = try XCTUnwrap(jsonString.data(using: .utf8))
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        XCTAssertEqual(json["id"] as? Int, 789)
        let dataDict = json["data"] as? [String: String]
        XCTAssertEqual(dataDict?["action"], "tap")
        XCTAssertEqual(dataDict?["source"], "notification")
    }

    func testReceivedNotificationDataToJSON() throws {
        let receivedData = ReceivedNotificationData(
            id: 101,
            title: "Push",
            body: "Push body",
            extra: ["key": "value"]
        )

        let jsonString = try receivedData.toJSONString()
        let data = try XCTUnwrap(jsonString.data(using: .utf8))
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        XCTAssertEqual(json["id"] as? Int, 101)
        XCTAssertEqual(json["title"] as? String, "Push")
        XCTAssertEqual(json["body"] as? String, "Push body")
        let extra = json["extra"] as? [String: String]
        XCTAssertEqual(extra?["key"], "value")
    }

    func testArrayToJSON() throws {
        let notifications = [
            ActiveNotification(id: 1, title: "First", body: "Body 1", sound: "", actionTypeId: "", attachments: nil),
            ActiveNotification(id: 2, title: "Second", body: "Body 2", sound: "", actionTypeId: "", attachments: nil)
        ]

        let jsonString = try notifications.toJSONString()
        let data = try XCTUnwrap(jsonString.data(using: .utf8))
        let json = try JSONSerialization.jsonObject(with: data) as! [[String: Any]]

        XCTAssertEqual(json.count, 2)
        XCTAssertEqual(json[0]["id"] as? Int, 1)
        XCTAssertEqual(json[1]["id"] as? Int, 2)
    }
}

// MARK: - NotificationHandler Tests

final class NotificationHandlerTests: XCTestCase {
    var handler: NotificationHandler!

    override func setUp() {
        super.setUp()
        handler = NotificationHandler()
    }

    override func tearDown() {
        handler = nil
        super.tearDown()
    }

    func testSaveAndRetrieveNotification() {
        let notification = makeTestNotification(id: 123, title: "Saved", body: "Body")
        handler.saveNotification("123", notification)

        // Create a mock request to test retrieval
        let content = UNMutableNotificationContent()
        content.title = "Saved"
        content.body = "Body"
        let request = UNNotificationRequest(identifier: "123", content: content, trigger: nil)

        let activeNotification = handler.toActiveNotification(request)
        XCTAssertNotNil(activeNotification)
        XCTAssertEqual(activeNotification?.id, 123)
        XCTAssertEqual(activeNotification?.title, "Saved")
    }

    func testToActiveNotificationReturnsNilForUnknown() {
        let content = UNMutableNotificationContent()
        content.title = "Unknown"
        let request = UNNotificationRequest(identifier: "unknown_id", content: content, trigger: nil)

        let activeNotification = handler.toActiveNotification(request)
        XCTAssertNil(activeNotification)
    }

    func testToPendingNotificationReturnsNilForUnknown() {
        let content = UNMutableNotificationContent()
        content.title = "Unknown"
        let request = UNNotificationRequest(identifier: "unknown_id", content: content, trigger: nil)

        let pendingNotification = handler.toPendingNotification(request)
        XCTAssertNil(pendingNotification)
    }

    func testToPendingNotificationWithSchedule() {
        let schedule = NotificationSchedule.every(interval: .hour, count: 1)
        let notification = makeTestNotification(id: 456, title: "Scheduled", schedule: schedule)
        handler.saveNotification("456", notification)

        let content = UNMutableNotificationContent()
        content.title = "Scheduled"
        let request = UNNotificationRequest(identifier: "456", content: content, trigger: nil)

        let pendingNotification = handler.toPendingNotification(request)
        XCTAssertNotNil(pendingNotification)
        XCTAssertEqual(pendingNotification?.id, 456)
        XCTAssertEqual(pendingNotification?.title, "Scheduled")
    }

    func testSetClickListenerActive() {
        // Initially false
        handler.setClickListenerActive(true)
        // Should not crash and store the state

        handler.setClickListenerActive(false)
        // Should not crash
    }
}
