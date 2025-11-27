// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.tauri.plugin.JSObject
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class NotificationInstrumentedTest {

    private lateinit var context: Context

    @Before
    fun setup() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
    }

    @Test
    fun testGetSound_withDefaultSound() {
        val notification = Notification()
        notification.sound = null

        // Use a valid resource ID as default
        val result = notification.getSound(context, 12345)

        assertNotNull(result)
        assertTrue(result!!.startsWith("android.resource://"))
        assertTrue(result.contains(context.packageName))
    }

    @Test
    fun testGetSound_customSoundNotFound_fallbackToDefault() {
        val notification = Notification()
        notification.sound = "nonexistent_sound.mp3"

        val result = notification.getSound(context, 54321)

        assertNotNull(result)
        assertTrue(result!!.contains("android.resource://"))
    }

    @Test
    fun testGetSound_nullDefaultSound() {
        val notification = Notification()
        notification.sound = null

        val result = notification.getSound(context, AssetUtils.RESOURCE_ID_ZERO_VALUE)

        assertNull(result)
    }

    @Test
    fun testGetIconColor_localColorTakesPrecedence() {
        val notification = Notification()
        notification.iconColor = "#FF0000"

        val result = notification.getIconColor("#00FF00")

        assertEquals("#FF0000", result)
    }

    @Test
    fun testGetIconColor_globalColorFallback() {
        val notification = Notification()
        notification.iconColor = null

        val result = notification.getIconColor("#00FF00")

        assertEquals("#00FF00", result)
    }

    @Test
    fun testGetSmallIcon_withDefaultIcon() {
        val notification = Notification()
        notification.icon = null

        val result = notification.getSmallIcon(context, android.R.drawable.ic_dialog_info)

        assertEquals(android.R.drawable.ic_dialog_info, result)
    }

    @Test
    fun testGetSmallIcon_customIconNotFound() {
        val notification = Notification()
        notification.icon = "nonexistent_icon"

        val result = notification.getSmallIcon(context, android.R.drawable.ic_dialog_alert)

        assertEquals(android.R.drawable.ic_dialog_alert, result)
    }

    @Test
    fun testGetLargeIcon_noIcon() {
        val notification = Notification()
        notification.largeIcon = null

        val result = notification.getLargeIcon(context)

        assertNull(result)
    }

    @Test
    fun testGetLargeIcon_customIcon() {
        val notification = Notification()
        notification.largeIcon = "ic_transparent"

        val result = notification.getLargeIcon(context)

        // May be null if resource doesn't exist, but should not crash
        // If it finds the resource, it should return a Bitmap
        assertTrue(result == null || result.width > 0)
    }

    @Test
    fun testBuildNotificationPendingList() {
        val notifications = listOf(
            Notification().apply {
                id = 1
                title = "First"
                body = "Body 1"
                schedule = NotificationSchedule.At()
                extra = JSObject()
            },
            Notification().apply {
                id = 2
                title = "Second"
                body = "Body 2"
                schedule = NotificationSchedule.Every().apply {
                    interval = NotificationInterval.Hour
                    count = 1
                }
                extra = JSObject()
            }
        )

        val pendingList = Notification.buildNotificationPendingList(notifications)

        assertEquals(2, pendingList.size)
        assertEquals(1, pendingList[0].id)
        assertEquals("First", pendingList[0].title)
        assertEquals(2, pendingList[1].id)
        assertEquals("Second", pendingList[1].title)
    }

    @Test
    fun testNotificationProperties() {
        val notification = Notification()
        notification.id = 100
        notification.title = "Test Title"
        notification.body = "Test Body"
        notification.largeBody = "Large body text"
        notification.summary = "Summary"
        notification.sound = "notification.mp3"
        notification.icon = "ic_notification"
        notification.largeIcon = "ic_large"
        notification.iconColor = "#FF5733"
        notification.actionTypeId = "action_type"
        notification.group = "test_group"
        notification.inboxLines = listOf("Line 1", "Line 2", "Line 3")
        notification.isGroupSummary = true
        notification.isOngoing = false
        notification.isAutoCancel = true
        notification.channelId = "test_channel"
        notification.visibility = 1
        notification.number = 5

        assertEquals(100, notification.id)
        assertEquals("Test Title", notification.title)
        assertEquals("Test Body", notification.body)
        assertEquals("Large body text", notification.largeBody)
        assertEquals("Summary", notification.summary)
        assertEquals("notification.mp3", notification.sound)
        assertEquals("ic_notification", notification.icon)
        assertEquals("ic_large", notification.largeIcon)
        assertEquals("#FF5733", notification.iconColor)
        assertEquals("action_type", notification.actionTypeId)
        assertEquals("test_group", notification.group)
        assertEquals(3, notification.inboxLines?.size)
        assertTrue(notification.isGroupSummary)
        assertFalse(notification.isOngoing)
        assertTrue(notification.isAutoCancel)
        assertEquals("test_channel", notification.channelId)
        assertEquals(1, notification.visibility)
        assertEquals(5, notification.number)
    }

    @Test
    fun testNotificationWithSchedule() {
        val schedule = NotificationSchedule.At().apply {
            repeating = true
            allowWhileIdle = false
        }

        val notification = Notification()
        notification.id = 200
        notification.title = "Scheduled"
        notification.schedule = schedule

        assertNotNull(notification.schedule)
        assertTrue((notification.schedule as NotificationSchedule.At).repeating)
        assertFalse((notification.schedule as NotificationSchedule.At).allowWhileIdle)
    }

    @Test
    fun testNotificationWithAttachments() {
        val attachment = NotificationAttachment()
        attachment.id = "image1"
        attachment.url = "file:///path/to/image.jpg"

        val notification = Notification()
        notification.id = 300
        notification.attachments = listOf(attachment)

        assertNotNull(notification.attachments)
        assertEquals(1, notification.attachments?.size)
        assertEquals("image1", notification.attachments?.get(0)?.id)
    }

    @Test
    fun testNotificationWithExtraData() {
        val extra = JSObject()
        extra.put("custom_key", "custom_value")
        extra.put("number", 42)

        val notification = Notification()
        notification.id = 400
        notification.extra = extra

        assertNotNull(notification.extra)
        assertEquals("custom_value", notification.extra?.getString("custom_key"))
        assertEquals(42, notification.extra?.getInt("number"))
    }

    @Test
    fun testPendingNotificationProperties() {
        val schedule = NotificationSchedule.Every().apply {
            interval = NotificationInterval.Day
            count = 1
        }
        val extra = JSObject()
        extra.put("key", "value")

        val pendingNotification = PendingNotification(
            id = 500,
            title = "Pending",
            body = "Pending body",
            schedule = schedule,
            extra = extra
        )

        assertEquals(500, pendingNotification.id)
        assertEquals("Pending", pendingNotification.title)
        assertEquals("Pending body", pendingNotification.body)
        assertNotNull(pendingNotification.schedule)
        assertNotNull(pendingNotification.extra)
    }

    @Test
    fun testNotificationInboxLines() {
        val notification = Notification()
        notification.id = 600
        notification.inboxLines = listOf(
            "Message from Alice",
            "Message from Bob",
            "Message from Charlie",
            "Message from David",
            "Message from Eve"
        )

        assertEquals(5, notification.inboxLines?.size)
        assertEquals("Message from Alice", notification.inboxLines?.get(0))
        assertEquals("Message from Eve", notification.inboxLines?.get(4))
    }

    @Test
    fun testNotificationGroupSummary() {
        val summary = Notification()
        summary.id = 700
        summary.group = "messages"
        summary.isGroupSummary = true
        summary.summary = "5 new messages"

        assertTrue(summary.isGroupSummary)
        assertEquals("messages", summary.group)
        assertEquals("5 new messages", summary.summary)
    }

    @Test
    fun testNotificationOngoing() {
        val notification = Notification()
        notification.id = 800
        notification.isOngoing = true
        notification.isAutoCancel = false

        assertTrue(notification.isOngoing)
        assertFalse(notification.isAutoCancel)
    }

    @Test
    fun testNotificationVisibility() {
        val visibilityLevels = listOf(
            -1, // Secret
            0,  // Private
            1   // Public
        )

        for (visibility in visibilityLevels) {
            val notification = Notification()
            notification.visibility = visibility

            assertEquals(visibility, notification.visibility)
        }
    }

    @Test
    fun testNotificationWithNullValues() {
        val notification = Notification()
        notification.id = 900

        assertNull(notification.title)
        assertNull(notification.body)
        assertNull(notification.largeBody)
        assertNull(notification.summary)
        assertNull(notification.sound)
        assertNull(notification.icon)
        assertNull(notification.largeIcon)
        assertNull(notification.iconColor)
        assertNull(notification.actionTypeId)
        assertNull(notification.group)
        assertNull(notification.inboxLines)
        assertNull(notification.extra)
        assertNull(notification.attachments)
        assertNull(notification.schedule)
        assertNull(notification.channelId)
        assertNull(notification.visibility)
        assertNull(notification.number)
    }
}
