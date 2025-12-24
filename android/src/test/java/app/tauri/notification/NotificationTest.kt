// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.content.Context
import android.content.res.Resources
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import app.tauri.plugin.JSObject
import io.mockk.*
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class NotificationTest {

    private lateinit var mockContext: Context
    private lateinit var mockResources: Resources

    @Before
    fun setup() {
        mockContext = mockk()
        mockResources = mockk()
        every { mockContext.resources } returns mockResources
        every { mockContext.packageName } returns "app.tauri.notification.test"
    }

    @Test
    fun testGetSound_customSound() {
        val notification = Notification()
        notification.sound = "custom_sound.mp3"

        every { mockResources.getIdentifier("custom_sound", "raw", "app.tauri.notification.test") } returns 12345

        val result = notification.getSound(mockContext, 999)

        assertEquals("android.resource://app.tauri.notification.test/12345", result)
    }

    @Test
    fun testGetSound_defaultSound() {
        val notification = Notification()
        notification.sound = null

        val result = notification.getSound(mockContext, 999)

        assertEquals("android.resource://app.tauri.notification.test/999", result)
    }

    @Test
    fun testGetSound_invalidCustomSound_fallbackToDefault() {
        val notification = Notification()
        notification.sound = "nonexistent_sound"

        every { mockResources.getIdentifier("nonexistent_sound", "raw", "app.tauri.notification.test") } returns 0

        val result = notification.getSound(mockContext, 999)

        assertEquals("android.resource://app.tauri.notification.test/999", result)
    }

    @Test
    fun testGetSound_soundWithPath() {
        val notification = Notification()
        notification.sound = "sounds/notification.wav"

        // AssetUtils.getResourceBaseName returns "notification.wav" for "sounds/notification.wav"
        // because it extracts after "/" but doesn't strip extension in that case
        every { mockResources.getIdentifier("notification.wav", "raw", "app.tauri.notification.test") } returns 54321

        val result = notification.getSound(mockContext, 999)

        assertEquals("android.resource://app.tauri.notification.test/54321", result)
    }

    @Test
    fun testGetSound_zeroDefaultSound() {
        val notification = Notification()
        notification.sound = null

        val result = notification.getSound(mockContext, 0)

        assertNull(result)
    }

    @Test
    fun testGetIconColor_localColor() {
        val notification = Notification()
        notification.iconColor = "#FF0000"

        val result = notification.getIconColor("#00FF00")

        assertEquals("#FF0000", result)
    }

    @Test
    fun testGetIconColor_globalColor() {
        val notification = Notification()
        notification.iconColor = null

        val result = notification.getIconColor("#00FF00")

        assertEquals("#00FF00", result)
    }

    @Test
    fun testGetSmallIcon_customIcon() {
        val notification = Notification()
        notification.icon = "custom_icon"

        every { mockResources.getIdentifier("custom_icon", "drawable", "app.tauri.notification.test") } returns 12345

        val result = notification.getSmallIcon(mockContext, 999)

        assertEquals(12345, result)
    }

    @Test
    fun testGetSmallIcon_defaultIcon() {
        val notification = Notification()
        notification.icon = null

        val result = notification.getSmallIcon(mockContext, 999)

        assertEquals(999, result)
    }

    @Test
    fun testGetSmallIcon_invalidCustomIcon_fallbackToDefault() {
        val notification = Notification()
        notification.icon = "nonexistent_icon"

        every { mockResources.getIdentifier("nonexistent_icon", "drawable", "app.tauri.notification.test") } returns 0

        val result = notification.getSmallIcon(mockContext, 999)

        assertEquals(999, result)
    }

    @Test
    fun testGetLargeIcon_validIcon() {
        mockkStatic(BitmapFactory::class)
        val mockBitmap = mockk<Bitmap>()

        val notification = Notification()
        notification.largeIcon = "large_icon"

        every { mockResources.getIdentifier("large_icon", "drawable", "app.tauri.notification.test") } returns 54321
        every { BitmapFactory.decodeResource(mockResources, 54321) } returns mockBitmap

        val result = notification.getLargeIcon(mockContext)

        assertNotNull(result)
        assertEquals(mockBitmap, result)

        unmockkStatic(BitmapFactory::class)
    }

    @Test
    fun testGetLargeIcon_nullIcon() {
        val notification = Notification()
        notification.largeIcon = null

        val result = notification.getLargeIcon(mockContext)

        assertNull(result)
    }

    @Test
    fun testBuildNotificationPendingList_singleNotification() {
        val notification = Notification()
        notification.id = 1
        notification.title = "Test Title"
        notification.body = "Test Body"
        notification.schedule = NotificationSchedule.At()
        notification.extra = JSObject()

        val pendingList = Notification.buildNotificationPendingList(listOf(notification))

        assertEquals(1, pendingList.size)
        assertEquals(1, pendingList[0].id)
        assertEquals("Test Title", pendingList[0].title)
        assertEquals("Test Body", pendingList[0].body)
        assertNotNull(pendingList[0].schedule)
        assertNotNull(pendingList[0].extra)
    }

    @Test
    fun testBuildNotificationPendingList_multipleNotifications() {
        val notification1 = Notification()
        notification1.id = 1
        notification1.title = "Title 1"
        notification1.body = "Body 1"

        val notification2 = Notification()
        notification2.id = 2
        notification2.title = "Title 2"
        notification2.body = "Body 2"

        val notification3 = Notification()
        notification3.id = 3
        notification3.title = "Title 3"
        notification3.body = "Body 3"

        val pendingList = Notification.buildNotificationPendingList(
            listOf(notification1, notification2, notification3)
        )

        assertEquals(3, pendingList.size)
        assertEquals(1, pendingList[0].id)
        assertEquals(2, pendingList[1].id)
        assertEquals(3, pendingList[2].id)
    }

    @Test
    fun testBuildNotificationPendingList_emptyList() {
        val pendingList = Notification.buildNotificationPendingList(emptyList())

        assertEquals(0, pendingList.size)
    }

    @Test
    fun testBuildNotificationPendingList_nullValues() {
        val notification = Notification()
        notification.id = 1
        notification.title = null
        notification.body = null
        notification.schedule = null
        notification.extra = null

        val pendingList = Notification.buildNotificationPendingList(listOf(notification))

        assertEquals(1, pendingList.size)
        assertEquals(1, pendingList[0].id)
        assertNull(pendingList[0].title)
        assertNull(pendingList[0].body)
        assertNull(pendingList[0].schedule)
        assertNull(pendingList[0].extra)
    }

    @Test
    fun testNotificationProperties() {
        val notification = Notification()
        notification.id = 42
        notification.title = "Title"
        notification.body = "Body"
        notification.largeBody = "Large Body"
        notification.summary = "Summary"
        notification.sound = "sound"
        notification.icon = "icon"
        notification.largeIcon = "large_icon"
        notification.iconColor = "#FF0000"
        notification.actionTypeId = "action_type"
        notification.group = "group"
        notification.inboxLines = listOf("Line 1", "Line 2")
        notification.isGroupSummary = true
        notification.isOngoing = true
        notification.isAutoCancel = false
        notification.extra = JSObject()
        notification.attachments = emptyList()
        notification.schedule = NotificationSchedule.At()
        notification.channelId = "channel"
        notification.sourceJson = "{}"
        notification.visibility = 1
        notification.number = 5

        assertEquals(42, notification.id)
        assertEquals("Title", notification.title)
        assertEquals("Body", notification.body)
        assertEquals("Large Body", notification.largeBody)
        assertEquals("Summary", notification.summary)
        assertEquals("sound", notification.sound)
        assertEquals("icon", notification.icon)
        assertEquals("large_icon", notification.largeIcon)
        assertEquals("#FF0000", notification.iconColor)
        assertEquals("action_type", notification.actionTypeId)
        assertEquals("group", notification.group)
        assertEquals(2, notification.inboxLines?.size)
        assertTrue(notification.isGroupSummary)
        assertTrue(notification.isOngoing)
        assertFalse(notification.isAutoCancel)
        assertNotNull(notification.extra)
        assertNotNull(notification.attachments)
        assertNotNull(notification.schedule)
        assertEquals("channel", notification.channelId)
        assertEquals("{}", notification.sourceJson)
        assertEquals(1, notification.visibility)
        assertEquals(5, notification.number)
    }

    @Test
    fun testPendingNotification_properties() {
        val schedule = NotificationSchedule.At()
        val extra = JSObject()

        val pendingNotification = PendingNotification(
            id = 10,
            title = "Pending Title",
            body = "Pending Body",
            schedule = schedule,
            extra = extra
        )

        assertEquals(10, pendingNotification.id)
        assertEquals("Pending Title", pendingNotification.title)
        assertEquals("Pending Body", pendingNotification.body)
        assertEquals(schedule, pendingNotification.schedule)
        assertEquals(extra, pendingNotification.extra)
    }
}
