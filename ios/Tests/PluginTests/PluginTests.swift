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
        let schedule = NotificationSchedule.every(interval: .day, count: 1)
        let pending = PendingNotification(id: 1, title: "Test", body: "Body", schedule: schedule)

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
            attachments: nil,
            source: "local"
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
        XCTAssertEqual(json?["source"] as? String, "local")
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

    // MARK: - NotificationHandler Tests

    func testNotificationHandlerToPendingNotificationReturnsNilForUnknown() {
        let handler = NotificationHandler()
        let content = UNMutableNotificationContent()
        content.title = "Test Title"
        content.body = "Test Body"

        let request = UNNotificationRequest(
            identifier: "123",
            content: content,
            trigger: nil
        )

        // Should return nil since no notification was saved with this identifier
        let pending = handler.toPendingNotification(request)
        XCTAssertNil(pending)
    }

    func testNotificationHandlerToPendingNotificationWithSavedNotification() {
        let handler = NotificationHandler()
        let schedule = NotificationSchedule.every(interval: .day, count: 1)
        let notification = Notification(
            id: 123,
            title: "Test Title",
            body: "Test Body",
            extra: nil,
            schedule: schedule,
            attachments: nil,
            sound: nil,
            group: nil,
            actionTypeId: nil,
            summary: nil,
            silent: nil
        )

        // Save the notification first
        handler.saveNotification("123", notification)

        let content = UNMutableNotificationContent()
        content.title = "Test Title"
        content.body = "Test Body"

        let request = UNNotificationRequest(
            identifier: "123",
            content: content,
            trigger: nil
        )

        let pending = handler.toPendingNotification(request)

        XCTAssertNotNil(pending)
        XCTAssertEqual(pending?.id, 123)
        XCTAssertEqual(pending?.title, "Test Title")
        XCTAssertEqual(pending?.body, "Test Body")
    }

    // MARK: - makeAttachments Tests

    func testMakeAttachmentsWithInvalidUrl() throws {
        let attachment = NotificationAttachment(
            id: "test-attachment",
            url: "",
            options: nil
        )

        XCTAssertThrowsError(try makeAttachments([attachment])) { error in
            if case NotificationError.attachmentFileNotFound(let path) = error {
                XCTAssertEqual(path, "")
            } else {
                XCTFail("Wrong error type")
            }
        }
    }

    // MARK: - Additional Schedule Tests

    func testHandleScheduledNotificationWithAtDate() throws {
        // Create a date in the future
        let dateFormatter = DateFormatter()
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.timeZone = TimeZone(secondsFromGMT: 0)
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"

        let futureDate = Date().addingTimeInterval(3600) // 1 hour from now
        let dateString = dateFormatter.string(from: futureDate)

        let schedule = NotificationSchedule.at(date: dateString, repeating: false)

        let trigger = try handleScheduledNotification(schedule)

        XCTAssertNotNil(trigger)
        XCTAssertTrue(trigger is UNTimeIntervalNotificationTrigger)

        if let timeTrigger = trigger as? UNTimeIntervalNotificationTrigger {
            XCTAssertFalse(timeTrigger.repeats)
            XCTAssertGreaterThan(timeTrigger.timeInterval, 3500)
            XCTAssertLessThan(timeTrigger.timeInterval, 3700)
        }
    }

    func testHandleScheduledNotificationWithPastDateThrows() {
        let dateFormatter = DateFormatter()
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.timeZone = TimeZone(secondsFromGMT: 0)
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"

        let pastDate = Date().addingTimeInterval(-3600) // 1 hour ago
        let dateString = dateFormatter.string(from: pastDate)

        let schedule = NotificationSchedule.at(date: dateString, repeating: false)

        XCTAssertThrowsError(try handleScheduledNotification(schedule)) { error in
            if case NotificationError.pastScheduledTime = error {
                // Expected error
            } else {
                XCTFail("Wrong error type")
            }
        }
    }

    func testHandleScheduledNotificationWithInvalidDateThrows() {
        let schedule = NotificationSchedule.at(date: "invalid-date", repeating: false)

        XCTAssertThrowsError(try handleScheduledNotification(schedule)) { error in
            if case NotificationError.invalidDate(let date) = error {
                XCTAssertEqual(date, "invalid-date")
            } else {
                XCTFail("Wrong error type")
            }
        }
    }

    // MARK: - Combined Options Tests

    func testMakeActionOptionsWithMultipleFlags() {
        let action = Action(
            id: "test",
            title: "Test",
            requiresAuthentication: true,
            foreground: true,
            destructive: true,
            input: nil,
            inputButtonTitle: nil,
            inputPlaceholder: nil
        )

        let options = makeActionOptions(action)

        // Should return foreground as it's checked first
        XCTAssertEqual(options, .foreground)
    }

    func testMakeCategoryOptionsWithMultipleFlags() {
        let actionType = ActionType(
            id: "test",
            actions: [],
            hiddenPreviewsBodyPlaceholder: nil,
            customDismissAction: true,
            allowInCarPlay: true,
            hiddenPreviewsShowTitle: true,
            hiddenPreviewsShowSubtitle: true,
            hiddenBodyPlaceholder: nil
        )

        let options = makeCategoryOptions(actionType)

        // Should return customDismissAction as it's checked first
        XCTAssertEqual(options, .customDismissAction)
    }

    // MARK: - Silent Notification Tests

    func testMakeNotificationContentWithSilentFlag() throws {
        let notification = Notification(
            id: 1,
            title: "Test",
            body: "Body",
            extra: nil,
            schedule: nil,
            attachments: nil,
            sound: nil,
            group: nil,
            actionTypeId: nil,
            summary: nil,
            silent: true
        )

        let content = try makeNotificationContent(notification)

        // Silent flag is handled in NotificationHandler.willPresent, not in content
        XCTAssertEqual(content.title, "Test")
        XCTAssertEqual(content.body, "Body")
    }

    // MARK: - Edge Case Tests

    func testMakeNotificationContentWithEmptyBody() throws {
        let notification = Notification(
            id: 1,
            title: "Test Title",
            body: nil,
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
        XCTAssertEqual(content.body, "")
    }

    func testMakeActionsWithTextInputActionWithoutButtonTitle() {
        let actions = [
            Action(
                id: "reply",
                title: "Reply",
                requiresAuthentication: nil,
                foreground: nil,
                destructive: nil,
                input: true,
                inputButtonTitle: nil,
                inputPlaceholder: "Type here..."
            )
        ]

        let result = makeActions(actions)

        XCTAssertEqual(result.count, 1)
        XCTAssertTrue(result[0] is UNTextInputNotificationAction)

        if let textAction = result[0] as? UNTextInputNotificationAction {
            XCTAssertEqual(textAction.identifier, "reply")
            XCTAssertEqual(textAction.title, "Reply")
        }
    }

    func testGetRepeatDateIntervalForYear() {
        let interval = getRepeatDateInterval(.year, 1)

        XCTAssertNotNil(interval)
        if let interval = interval {
            // Year duration varies, check it's approximately 365 days
            XCTAssertGreaterThan(interval.duration, 364 * 24 * 60 * 60)
            XCTAssertLessThan(interval.duration, 366 * 24 * 60 * 60)
        }
    }

    func testGetRepeatDateIntervalForMultipleUnits() {
        let interval = getRepeatDateInterval(.hour, 3)

        XCTAssertNotNil(interval)
        if let interval = interval {
            XCTAssertEqual(interval.duration, 3 * 60 * 60, accuracy: 1.0)
        }
    }

    func testMakeCategoryOptionsWithHiddenPreviewsShowSubtitle() {
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

    func testMakeActionOptionsWithNoFlags() {
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

    func testMakeCategoryOptionsWithNoFlags() {
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

    // MARK: - NotificationSchedule Decoding Tests

    func testNotificationScheduleDecodingAtDate() throws {
        let json = """
        {
            "at": {
                "date": "2024-12-25T10:30:00.000Z",
                "repeating": false
            }
        }
        """

        let decoder = JSONDecoder()
        let schedule = try decoder.decode(NotificationSchedule.self, from: json.data(using: .utf8)!)

        if case .at(let date, let repeating) = schedule {
            XCTAssertEqual(date, "2024-12-25T10:30:00.000Z")
            XCTAssertFalse(repeating)
        } else {
            XCTFail("Wrong schedule type")
        }
    }

    func testNotificationScheduleDecodingInterval() throws {
        let json = """
        {
            "interval": {
                "interval": {
                    "hour": 9,
                    "minute": 30
                }
            }
        }
        """

        let decoder = JSONDecoder()
        let schedule = try decoder.decode(NotificationSchedule.self, from: json.data(using: .utf8)!)

        if case .interval(let interval) = schedule {
            XCTAssertEqual(interval.hour, 9)
            XCTAssertEqual(interval.minute, 30)
        } else {
            XCTFail("Wrong schedule type")
        }
    }

    func testNotificationScheduleDecodingEvery() throws {
        let json = """
        {
            "every": {
                "interval": "minute",
                "count": 5
            }
        }
        """

        let decoder = JSONDecoder()
        let schedule = try decoder.decode(NotificationSchedule.self, from: json.data(using: .utf8)!)

        if case .every(let interval, let count) = schedule {
            XCTAssertEqual(interval, .minute)
            XCTAssertEqual(count, 5)
        } else {
            XCTFail("Wrong schedule type")
        }
    }

    // MARK: - Additional Coverage Tests

    func testNotificationDecodingWithAllFields() throws {
        let json = """
        {
            "id": 1,
            "title": "Test Title",
            "body": "Test Body",
            "extra": {"key": "value"},
            "sound": "custom.wav",
            "group": "test-group",
            "actionTypeId": "TEST_CATEGORY",
            "summary": "Summary",
            "silent": true
        }
        """

        let decoder = JSONDecoder()
        let notification = try decoder.decode(Notification.self, from: json.data(using: .utf8)!)

        XCTAssertEqual(notification.id, 1)
        XCTAssertEqual(notification.title, "Test Title")
        XCTAssertEqual(notification.body, "Test Body")
        XCTAssertEqual(notification.extra?["key"], "value")
        XCTAssertEqual(notification.sound, "custom.wav")
        XCTAssertEqual(notification.group, "test-group")
        XCTAssertEqual(notification.actionTypeId, "TEST_CATEGORY")
        XCTAssertEqual(notification.summary, "Summary")
        XCTAssertEqual(notification.silent, true)
    }

    func testActionDecodingWithAllFields() throws {
        let json = """
        {
            "id": "reply",
            "title": "Reply",
            "requiresAuthentication": true,
            "foreground": true,
            "destructive": false,
            "input": true,
            "inputButtonTitle": "Send",
            "inputPlaceholder": "Type here..."
        }
        """

        let decoder = JSONDecoder()
        let action = try decoder.decode(Action.self, from: json.data(using: .utf8)!)

        XCTAssertEqual(action.id, "reply")
        XCTAssertEqual(action.title, "Reply")
        XCTAssertEqual(action.requiresAuthentication, true)
        XCTAssertEqual(action.foreground, true)
        XCTAssertEqual(action.destructive, false)
        XCTAssertEqual(action.input, true)
        XCTAssertEqual(action.inputButtonTitle, "Send")
        XCTAssertEqual(action.inputPlaceholder, "Type here...")
    }

    func testActionTypeDecodingWithAllFields() throws {
        let json = """
        {
            "id": "TEST_CATEGORY",
            "actions": [
                {
                    "id": "action1",
                    "title": "Action 1"
                }
            ],
            "hiddenPreviewsBodyPlaceholder": "Hidden",
            "customDismissAction": true,
            "allowInCarPlay": true,
            "hiddenPreviewsShowTitle": true,
            "hiddenPreviewsShowSubtitle": true,
            "hiddenBodyPlaceholder": "Body Hidden"
        }
        """

        let decoder = JSONDecoder()
        let actionType = try decoder.decode(ActionType.self, from: json.data(using: .utf8)!)

        XCTAssertEqual(actionType.id, "TEST_CATEGORY")
        XCTAssertEqual(actionType.actions.count, 1)
        XCTAssertEqual(actionType.hiddenPreviewsBodyPlaceholder, "Hidden")
        XCTAssertEqual(actionType.customDismissAction, true)
        XCTAssertEqual(actionType.allowInCarPlay, true)
        XCTAssertEqual(actionType.hiddenPreviewsShowTitle, true)
        XCTAssertEqual(actionType.hiddenPreviewsShowSubtitle, true)
        XCTAssertEqual(actionType.hiddenBodyPlaceholder, "Body Hidden")
    }

    func testScheduleEveryKindDecoding() throws {
        let kinds = ["year", "month", "twoWeeks", "week", "day", "hour", "minute", "second"]
        let expected: [ScheduleEveryKind] = [.year, .month, .twoWeeks, .week, .day, .hour, .minute, .second]

        for (index, kind) in kinds.enumerated() {
            let json = "\"\(kind)\""
            let decoder = JSONDecoder()
            let decoded = try decoder.decode(ScheduleEveryKind.self, from: json.data(using: .utf8)!)
            XCTAssertEqual(decoded, expected[index])
        }
    }

    func testNotificationAttachmentDecoding() throws {
        let json = """
        {
            "id": "attachment1",
            "url": "https://example.com/image.jpg",
            "options": {
                "iosUNNotificationAttachmentOptionsTypeHintKey": "public.jpeg",
                "iosUNNotificationAttachmentOptionsThumbnailHiddenKey": "true"
            }
        }
        """

        let decoder = JSONDecoder()
        let attachment = try decoder.decode(NotificationAttachment.self, from: json.data(using: .utf8)!)

        XCTAssertEqual(attachment.id, "attachment1")
        XCTAssertEqual(attachment.url, "https://example.com/image.jpg")
        XCTAssertEqual(attachment.options?.iosUNNotificationAttachmentOptionsTypeHintKey, "public.jpeg")
        XCTAssertEqual(attachment.options?.iosUNNotificationAttachmentOptionsThumbnailHiddenKey, "true")
    }

    // MARK: - Push Notification Foreground Path Serialization Tests

    /// Tests encoding of ReceivedNotificationData — the struct used in the foreground push
    /// notification path (NotificationHandler.willPresent → toReceivedNotification).
    /// This reproduces the exact serialization that Channel.send<T: Encodable>() performs.
    func testReceivedNotificationDataEncoding() throws {
        let data = ReceivedNotificationData(
            id: 42,
            title: "Push Title",
            body: "Push Body",
            extra: ["key": "value", "another": "data"],
            source: "push"
        )

        let encoder = JSONEncoder()
        let jsonData = try encoder.encode(data)

        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        XCTAssertNotNil(json)
        XCTAssertEqual(json?["id"] as? Int, 42)
        XCTAssertEqual(json?["title"] as? String, "Push Title")
        XCTAssertEqual(json?["body"] as? String, "Push Body")
        XCTAssertEqual(json?["source"] as? String, "push")

        let extra = json?["extra"] as? [String: String]
        XCTAssertNotNil(extra)
        XCTAssertEqual(extra?["key"], "value")
        XCTAssertEqual(extra?["another"], "data")
    }

    func testReceivedNotificationDataEncodingWithNilExtra() throws {
        let data = ReceivedNotificationData(
            id: 1,
            title: "Title",
            body: "Body",
            extra: nil,
            source: "push"
        )

        let encoder = JSONEncoder()
        let jsonData = try encoder.encode(data)

        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        XCTAssertNotNil(json)
        XCTAssertEqual(json?["id"] as? Int, 1)
        XCTAssertEqual(json?["source"] as? String, "push")
        // extra should be absent or null
        XCTAssertTrue(json?["extra"] == nil || json?["extra"] is NSNull)
    }

    /// Tests encoding of NotificationClickedData — used for notification click events.
    func testNotificationClickedDataEncoding() throws {
        let data = NotificationClickedData(
            id: 7,
            data: ["action": "open", "screen": "home"]
        )

        let encoder = JSONEncoder()
        let jsonData = try encoder.encode(data)

        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        XCTAssertNotNil(json)
        XCTAssertEqual(json?["id"] as? Int, 7)

        let clickData = json?["data"] as? [String: String]
        XCTAssertNotNil(clickData)
        XCTAssertEqual(clickData?["action"], "open")
        XCTAssertEqual(clickData?["screen"], "home")
    }

    func testNotificationClickedDataEncodingWithNilData() throws {
        let data = NotificationClickedData(
            id: -1,
            data: nil
        )

        let encoder = JSONEncoder()
        let jsonData = try encoder.encode(data)

        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        XCTAssertNotNil(json)
        XCTAssertEqual(json?["id"] as? Int, -1)
        XCTAssertTrue(json?["data"] == nil || json?["data"] is NSNull)
    }

    func testMakeNotificationContentWithAttachmentsError() throws {
        let notification = Notification(
            id: 1,
            title: "Test",
            body: "Body",
            extra: nil,
            schedule: nil,
            attachments: [
                NotificationAttachment(
                    id: "test",
                    url: "",
                    options: nil
                )
            ],
            sound: nil,
            group: nil,
            actionTypeId: nil,
            summary: nil,
            silent: nil
        )

        XCTAssertThrowsError(try makeNotificationContent(notification))
    }
}
