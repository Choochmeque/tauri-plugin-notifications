// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import XCTest
import UserNotifications
@testable import tauri_plugin_notifications

final class NotificationTests: XCTestCase {

    // MARK: - Notification Content Tests

    func testMakeNotificationContentWithBasicNotification() throws {
        let notification = Notification(
            id: 1,
            title: "Test Title",
            body: "Test Body",
            extra: nil,
            schedule: nil,
            attachments: nil,
            sound: nil,
            group: nil,
            actionTypeId: nil,
            summary: nil,
            silent: nil
        )

        let content = try makeNotificationContent(notification)

        XCTAssertEqual(content.title, "Test Title")
        XCTAssertEqual(content.body, "Test Body")
        XCTAssertTrue(content.userInfo.isEmpty)
    }

    func testMakeNotificationContentWithExtra() throws {
        let notification = Notification(
            id: 1,
            title: "Test",
            body: "Body",
            extra: ["key1": "value1", "key2": "value2"],
            schedule: nil,
            attachments: nil,
            sound: nil,
            group: nil,
            actionTypeId: nil,
            summary: nil,
            silent: nil
        )

        let content = try makeNotificationContent(notification)

        if let extra = content.userInfo["__EXTRA__"] as? [String: String] {
            XCTAssertEqual(extra["key1"], "value1")
            XCTAssertEqual(extra["key2"], "value2")
        } else {
            XCTFail("Extra data not found in userInfo")
        }
    }

    func testMakeNotificationContentWithActionTypeId() throws {
        let notification = Notification(
            id: 1,
            title: "Test",
            body: nil,
            extra: nil,
            schedule: nil,
            attachments: nil,
            sound: nil,
            group: nil,
            actionTypeId: "TEST_CATEGORY",
            summary: nil,
            silent: nil
        )

        let content = try makeNotificationContent(notification)

        XCTAssertEqual(content.categoryIdentifier, "TEST_CATEGORY")
    }

    func testMakeNotificationContentWithThreadIdentifier() throws {
        let notification = Notification(
            id: 1,
            title: "Test",
            body: nil,
            extra: nil,
            schedule: nil,
            attachments: nil,
            sound: nil,
            group: "test-group",
            actionTypeId: nil,
            summary: nil,
            silent: nil
        )

        let content = try makeNotificationContent(notification)

        XCTAssertEqual(content.threadIdentifier, "test-group")
    }

    func testMakeNotificationContentWithSummary() throws {
        let notification = Notification(
            id: 1,
            title: "Test",
            body: nil,
            extra: nil,
            schedule: nil,
            attachments: nil,
            sound: nil,
            group: nil,
            actionTypeId: nil,
            summary: "Test Summary",
            silent: nil
        )

        let content = try makeNotificationContent(notification)

        XCTAssertEqual(content.summaryArgument, "Test Summary")
    }

    func testMakeNotificationContentWithSound() throws {
        let notification = Notification(
            id: 1,
            title: "Test",
            body: nil,
            extra: nil,
            schedule: nil,
            attachments: nil,
            sound: "custom_sound.wav",
            group: nil,
            actionTypeId: nil,
            summary: nil,
            silent: nil
        )

        let content = try makeNotificationContent(notification)

        XCTAssertNotNil(content.sound)
    }

    // MARK: - Attachment Tests

    func testMakeAttachmentUrl() {
        let url = makeAttachmentUrl("https://example.com/image.jpg")
        XCTAssertNotNil(url)
        XCTAssertEqual(url?.absoluteString, "https://example.com/image.jpg")
    }

    func testMakeAttachmentUrlWithInvalidPath() {
        let url = makeAttachmentUrl("")
        XCTAssertNil(url)
    }

    func testMakeAttachmentOptions() {
        let options = NotificationAttachmentOptions(
            iosUNNotificationAttachmentOptionsTypeHintKey: "public.jpeg",
            iosUNNotificationAttachmentOptionsThumbnailHiddenKey: "true",
            iosUNNotificationAttachmentOptionsThumbnailClippingRectKey: nil,
            iosUNNotificationAttachmentOptionsThumbnailTimeKey: nil
        )

        let result = makeAttachmentOptions(options)

        XCTAssertEqual(result[UNNotificationAttachmentOptionsTypeHintKey] as? String, "public.jpeg")
        XCTAssertEqual(result[UNNotificationAttachmentOptionsThumbnailHiddenKey] as? String, "true")
    }

    // MARK: - Schedule Tests

    func testGetDateComponents() {
        let interval = ScheduleInterval(
            year: 2024,
            month: 12,
            day: 25,
            weekday: nil,
            hour: 10,
            minute: 30,
            second: 0
        )

        let dateComponents = getDateComponents(interval)

        XCTAssertEqual(dateComponents.year, 2024)
        XCTAssertEqual(dateComponents.month, 12)
        XCTAssertEqual(dateComponents.day, 25)
        XCTAssertEqual(dateComponents.hour, 10)
        XCTAssertEqual(dateComponents.minute, 30)
        XCTAssertEqual(dateComponents.second, 0)
        XCTAssertNil(dateComponents.weekday)
    }

    func testGetDateComponentsWithWeekday() {
        let interval = ScheduleInterval(
            year: nil,
            month: nil,
            day: nil,
            weekday: 2,
            hour: 9,
            minute: 0,
            second: nil
        )

        let dateComponents = getDateComponents(interval)

        XCTAssertNil(dateComponents.year)
        XCTAssertNil(dateComponents.month)
        XCTAssertEqual(dateComponents.weekday, 2)
        XCTAssertEqual(dateComponents.hour, 9)
        XCTAssertEqual(dateComponents.minute, 0)
    }

    func testGetRepeatDateIntervalForMinutes() {
        let interval = getRepeatDateInterval(.minute, 5)

        XCTAssertNotNil(interval)
        if let interval = interval {
            XCTAssertEqual(interval.duration, 5 * 60, accuracy: 1.0)
        }
    }

    func testGetRepeatDateIntervalForHours() {
        let interval = getRepeatDateInterval(.hour, 2)

        XCTAssertNotNil(interval)
        if let interval = interval {
            XCTAssertEqual(interval.duration, 2 * 60 * 60, accuracy: 1.0)
        }
    }

    func testGetRepeatDateIntervalForDays() {
        let interval = getRepeatDateInterval(.day, 1)

        XCTAssertNotNil(interval)
        if let interval = interval {
            XCTAssertEqual(interval.duration, 24 * 60 * 60, accuracy: 1.0)
        }
    }

    func testGetRepeatDateIntervalForWeeks() {
        let interval = getRepeatDateInterval(.week, 1)

        XCTAssertNotNil(interval)
        if let interval = interval {
            XCTAssertEqual(interval.duration, 7 * 24 * 60 * 60, accuracy: 1.0)
        }
    }

    func testGetRepeatDateIntervalForTwoWeeks() {
        let interval = getRepeatDateInterval(.twoWeeks, 1)

        XCTAssertNotNil(interval)
        if let interval = interval {
            XCTAssertEqual(interval.duration, 14 * 24 * 60 * 60, accuracy: 1.0)
        }
    }

    func testGetRepeatDateIntervalForMonths() {
        let interval = getRepeatDateInterval(.month, 1)

        XCTAssertNotNil(interval)
        if let interval = interval {
            // Month duration varies, so just check it's approximately 30 days
            XCTAssertGreaterThan(interval.duration, 28 * 24 * 60 * 60)
            XCTAssertLessThan(interval.duration, 32 * 24 * 60 * 60)
        }
    }

    func testHandleScheduledNotificationWithEveryMinute() throws {
        let schedule = NotificationSchedule.every(interval: .minute, count: 2)

        let trigger = try handleScheduledNotification(schedule)

        XCTAssertNotNil(trigger)
        XCTAssertTrue(trigger is UNTimeIntervalNotificationTrigger)

        if let timeTrigger = trigger as? UNTimeIntervalNotificationTrigger {
            XCTAssertTrue(timeTrigger.repeats)
            XCTAssertEqual(timeTrigger.timeInterval, 120, accuracy: 1.0)
        }
    }

    func testHandleScheduledNotificationWithIntervalThrowsForShortInterval() {
        let schedule = NotificationSchedule.every(interval: .second, count: 30)

        XCTAssertThrowsError(try handleScheduledNotification(schedule)) { error in
            XCTAssertTrue(error is NotificationError)
            if case NotificationError.triggerRepeatIntervalTooShort = error {
                // Expected error
            } else {
                XCTFail("Wrong error type")
            }
        }
    }

    func testHandleScheduledNotificationWithInterval() throws {
        let interval = ScheduleInterval(
            year: nil,
            month: nil,
            day: nil,
            weekday: 2,
            hour: 9,
            minute: 0,
            second: 0
        )
        let schedule = NotificationSchedule.interval(interval: interval)

        let trigger = try handleScheduledNotification(schedule)

        XCTAssertNotNil(trigger)
        XCTAssertTrue(trigger is UNCalendarNotificationTrigger)

        if let calendarTrigger = trigger as? UNCalendarNotificationTrigger {
            XCTAssertTrue(calendarTrigger.repeats)
            XCTAssertEqual(calendarTrigger.dateComponents.weekday, 2)
            XCTAssertEqual(calendarTrigger.dateComponents.hour, 9)
        }
    }

    // MARK: - Category and Action Tests

    func testMakeActionOptionsWithForeground() {
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

    func testMakeActionOptionsWithDestructive() {
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

    func testMakeActionOptionsWithAuthRequired() {
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

    func testMakeCategoryOptionsWithCustomDismiss() {
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

    func testMakeCategoryOptionsWithCarPlay() {
        let actionType = ActionType(
            id: "test",
            actions: [],
            hiddenPreviewsBodyPlaceholder: nil,
            customDismissAction: nil,
            allowInCarPlay: true,
            hiddenPreviewsShowTitle: nil,
            hiddenPreviewsShowSubtitle: nil,
            hiddenBodyPlaceholder: nil
        )

        let options = makeCategoryOptions(actionType)

        XCTAssertEqual(options, .allowInCarPlay)
    }

    func testMakeCategoryOptionsWithHiddenPreviewsShowTitle() {
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

    func testMakeActionsCreatesBasicAction() {
        let actions = [
            Action(
                id: "action1",
                title: "Action 1",
                requiresAuthentication: nil,
                foreground: nil,
                destructive: nil,
                input: nil,
                inputButtonTitle: nil,
                inputPlaceholder: nil
            )
        ]

        let result = makeActions(actions)

        XCTAssertEqual(result.count, 1)
        XCTAssertEqual(result[0].identifier, "action1")
        XCTAssertEqual(result[0].title, "Action 1")
        XCTAssertFalse(result[0] is UNTextInputNotificationAction)
    }

    func testMakeActionsCreatesTextInputAction() {
        let actions = [
            Action(
                id: "reply",
                title: "Reply",
                requiresAuthentication: nil,
                foreground: nil,
                destructive: nil,
                input: true,
                inputButtonTitle: "Send",
                inputPlaceholder: "Type your reply..."
            )
        ]

        let result = makeActions(actions)

        XCTAssertEqual(result.count, 1)
        XCTAssertTrue(result[0] is UNTextInputNotificationAction)

        if let textAction = result[0] as? UNTextInputNotificationAction {
            XCTAssertEqual(textAction.identifier, "reply")
            XCTAssertEqual(textAction.title, "Reply")
            XCTAssertEqual(textAction.textInputButtonTitle, "Send")
            XCTAssertEqual(textAction.textInputPlaceholder, "Type your reply...")
        }
    }

    func testMakeActionsCreatesMultipleActions() {
        let actions = [
            Action(id: "action1", title: "Action 1", requiresAuthentication: nil, foreground: nil, destructive: nil, input: nil, inputButtonTitle: nil, inputPlaceholder: nil),
            Action(id: "action2", title: "Action 2", requiresAuthentication: nil, foreground: true, destructive: nil, input: nil, inputButtonTitle: nil, inputPlaceholder: nil),
            Action(id: "action3", title: "Action 3", requiresAuthentication: nil, foreground: nil, destructive: true, input: nil, inputButtonTitle: nil, inputPlaceholder: nil)
        ]

        let result = makeActions(actions)

        XCTAssertEqual(result.count, 3)
        XCTAssertEqual(result[0].identifier, "action1")
        XCTAssertEqual(result[1].identifier, "action2")
        XCTAssertEqual(result[2].identifier, "action3")
    }

    // MARK: - Error Tests

    func testNotificationErrorDescriptions() {
        let error1 = NotificationError.triggerRepeatIntervalTooShort
        XCTAssertEqual(error1.errorDescription, "Schedule interval too short, must be a least 1 minute")

        let error2 = NotificationError.attachmentFileNotFound(path: "/path/to/file")
        XCTAssertEqual(error2.errorDescription, "Unable to find file /path/to/file for attachment")

        let error3 = NotificationError.attachmentUnableToCreate("Test error")
        XCTAssertEqual(error3.errorDescription, "Failed to create attachment: Test error")

        let error4 = NotificationError.pastScheduledTime
        XCTAssertEqual(error4.errorDescription, "Scheduled time must be *after* current time")

        let error5 = NotificationError.invalidDate("2024-13-32")
        XCTAssertEqual(error5.errorDescription, "Could not parse date 2024-13-32")
    }

    // MARK: - Data Structure Tests

    func testPendingNotificationEncoding() throws {
        let pending = PendingNotification(id: 1, title: "Test", body: "Body")

        let encoder = JSONEncoder()
        let data = try encoder.encode(pending)

        XCTAssertFalse(data.isEmpty)

        // Verify the JSON structure
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        XCTAssertNotNil(json)
        XCTAssertEqual(json?["id"] as? Int, 1)
        XCTAssertEqual(json?["title"] as? String, "Test")
        XCTAssertEqual(json?["body"] as? String, "Body")
    }

    func testActiveNotificationEncoding() throws {
        let active = ActiveNotification(
            id: 1,
            title: "Test",
            body: "Body",
            sound: "default",
            actionTypeId: "CATEGORY",
            attachments: nil
        )

        let encoder = JSONEncoder()
        let data = try encoder.encode(active)

        XCTAssertFalse(data.isEmpty)

        // Verify the JSON structure
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        XCTAssertNotNil(json)
        XCTAssertEqual(json?["id"] as? Int, 1)
        XCTAssertEqual(json?["title"] as? String, "Test")
        XCTAssertEqual(json?["body"] as? String, "Body")
        XCTAssertEqual(json?["sound"] as? String, "default")
        XCTAssertEqual(json?["actionTypeId"] as? String, "CATEGORY")
    }

    func testReceivedNotificationEncoding() throws {
        let active = ActiveNotification(
            id: 1,
            title: "Test",
            body: "Body",
            sound: "default",
            actionTypeId: "CATEGORY",
            attachments: nil
        )

        let received = ReceivedNotification(
            actionId: "tap",
            inputValue: nil,
            notification: active
        )

        let encoder = JSONEncoder()
        let data = try encoder.encode(received)

        XCTAssertFalse(data.isEmpty)

        // Verify the JSON structure
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        XCTAssertNotNil(json)
        XCTAssertEqual(json?["actionId"] as? String, "tap")
        XCTAssertNil(json?["inputValue"])

        let notification = json?["notification"] as? [String: Any]
        XCTAssertNotNil(notification)
        XCTAssertEqual(notification?["id"] as? Int, 1)
    }
}
