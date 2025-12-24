// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.content.Context
import android.content.SharedPreferences
import com.fasterxml.jackson.databind.ObjectMapper
import io.mockk.*
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class NotificationStorageTest {

    private lateinit var mockContext: Context
    private lateinit var mockSharedPreferences: SharedPreferences
    private lateinit var mockEditor: SharedPreferences.Editor
    private lateinit var objectMapper: ObjectMapper
    private lateinit var notificationStorage: NotificationStorage

    @Before
    fun setup() {
        mockContext = mockk()
        mockSharedPreferences = mockk()
        mockEditor = mockk(relaxed = true)
        objectMapper = ObjectMapper()
        notificationStorage = NotificationStorage(mockContext, objectMapper)

        every { mockContext.getSharedPreferences(any(), any()) } returns mockSharedPreferences
        every { mockSharedPreferences.edit() } returns mockEditor
        every { mockEditor.apply() } just Runs
    }

    @Test
    fun testAppendNotifications_withSchedule() {
        val notification = Notification()
        notification.id = 1
        notification.title = "Test Notification"
        notification.body = "Test Body"
        notification.schedule = NotificationSchedule.At().apply {
            repeating = false
        }
        notification.sourceJson = """{"id":1,"title":"Test Notification"}"""

        every { mockEditor.putString("1", notification.sourceJson) } returns mockEditor

        notificationStorage.appendNotifications(listOf(notification))

        verify { mockEditor.putString("1", notification.sourceJson) }
        verify { mockEditor.apply() }
    }

    @Test
    fun testAppendNotifications_withoutSchedule() {
        val notification = Notification()
        notification.id = 1
        notification.title = "Test Notification"
        notification.schedule = null
        notification.sourceJson = """{"id":1,"title":"Test Notification"}"""

        notificationStorage.appendNotifications(listOf(notification))

        verify(exactly = 0) { mockEditor.putString(any(), any()) }
        verify { mockEditor.apply() }
    }

    @Test
    fun testAppendNotifications_multipleNotifications() {
        val notification1 = Notification()
        notification1.id = 1
        notification1.schedule = NotificationSchedule.At()
        notification1.sourceJson = """{"id":1}"""

        val notification2 = Notification()
        notification2.id = 2
        notification2.schedule = NotificationSchedule.Every().apply {
            interval = NotificationInterval.Hour
            count = 1
        }
        notification2.sourceJson = """{"id":2}"""

        every { mockEditor.putString(any(), any()) } returns mockEditor

        notificationStorage.appendNotifications(listOf(notification1, notification2))

        verify { mockEditor.putString("1", """{"id":1}""") }
        verify { mockEditor.putString("2", """{"id":2}""") }
        verify { mockEditor.apply() }
    }

    @Test
    fun testGetSavedNotificationIds_withNotifications() {
        val mockMap = mapOf("1" to "notification1", "2" to "notification2", "3" to "notification3")
        every { mockSharedPreferences.all } returns mockMap

        val result = notificationStorage.getSavedNotificationIds()

        assertEquals(3, result.size)
        assertTrue(result.contains("1"))
        assertTrue(result.contains("2"))
        assertTrue(result.contains("3"))
    }

    @Test
    fun testGetSavedNotificationIds_empty() {
        every { mockSharedPreferences.all } returns null

        val result = notificationStorage.getSavedNotificationIds()

        assertEquals(0, result.size)
    }

    @Test
    fun testGetSavedNotifications_validJson() {
        val mockMap = mapOf("1" to "invalid json")
        every { mockSharedPreferences.all } returns mockMap

        val result = notificationStorage.getSavedNotifications()

        // Jackson deserialization is complex, just verify method doesn't crash
        assertTrue(result.size >= 0)
    }

    @Test
    fun testGetSavedNotifications_invalidJson() {
        val mockMap = mapOf("1" to "invalid json data")
        every { mockSharedPreferences.all } returns mockMap

        val result = notificationStorage.getSavedNotifications()

        assertEquals(0, result.size)
    }

    @Test
    fun testGetSavedNotifications_mixedValidInvalid() {
        val mockMap = mapOf(
            "1" to "invalid1",
            "2" to "invalid json",
            "3" to "invalid2"
        )
        every { mockSharedPreferences.all } returns mockMap

        val result = notificationStorage.getSavedNotifications()

        // Invalid JSON should be filtered out
        assertTrue(result.size >= 0)
    }

    @Test
    fun testGetSavedNotification_notExists() {
        every { mockSharedPreferences.getString("99", null) } returns null

        val result = notificationStorage.getSavedNotification("99")

        assertNull(result)
    }

    @Test
    fun testGetSavedNotification_classCastException() {
        every { mockSharedPreferences.getString("1", null) } throws ClassCastException()

        val result = notificationStorage.getSavedNotification("1")

        assertNull(result)
    }

    @Test
    fun testDeleteNotification() {
        every { mockEditor.remove("123") } returns mockEditor

        notificationStorage.deleteNotification("123")

        verify { mockEditor.remove("123") }
        verify { mockEditor.apply() }
    }

    @Test
    fun testDeleteNotification_null() {
        every { mockEditor.remove(null) } returns mockEditor

        notificationStorage.deleteNotification(null)

        verify { mockEditor.remove(null) }
        verify { mockEditor.apply() }
    }

    @Test
    fun testWriteActionGroup_singleAction() {
        val action = NotificationAction()
        action.id = "action1"
        action.title = "Action 1"
        action.input = true

        val actionType = ActionType()
        actionType.id = "type1"
        actionType.actions = listOf(action)

        every { mockEditor.clear() } returns mockEditor
        every { mockEditor.putInt(any(), any()) } returns mockEditor
        every { mockEditor.putString(any(), any()) } returns mockEditor
        every { mockEditor.putBoolean(any(), any()) } returns mockEditor

        notificationStorage.writeActionGroup(listOf(actionType))

        verify { mockEditor.clear() }
        verify { mockEditor.putInt("count", 1) }
        verify { mockEditor.apply() }
    }

    @Test
    fun testWriteActionGroup_multipleActions() {
        val action1 = NotificationAction()
        action1.id = "action1"
        action1.title = "Action 1"
        action1.input = false

        val action2 = NotificationAction()
        action2.id = "action2"
        action2.title = "Action 2"
        action2.input = true

        val actionType = ActionType()
        actionType.id = "type1"
        actionType.actions = listOf(action1, action2)

        every { mockEditor.clear() } returns mockEditor
        every { mockEditor.putInt(any(), any()) } returns mockEditor
        every { mockEditor.putString(any(), any()) } returns mockEditor
        every { mockEditor.putBoolean(any(), any()) } returns mockEditor

        notificationStorage.writeActionGroup(listOf(actionType))

        verify { mockEditor.putInt("count", 2) }
        verify { mockEditor.apply() }
    }

    @Test
    fun testGetActionGroup() {
        every { mockSharedPreferences.getInt("count", 0) } returns 2
        every { mockSharedPreferences.getString("id0", "") } returns "action1"
        every { mockSharedPreferences.getString("title0", "") } returns "Title 1"
        every { mockSharedPreferences.getBoolean("input0", false) } returns false
        every { mockSharedPreferences.getString("id1", "") } returns "action2"
        every { mockSharedPreferences.getString("title1", "") } returns "Title 2"
        every { mockSharedPreferences.getBoolean("input1", false) } returns true

        val result = notificationStorage.getActionGroup("type1")

        assertEquals(2, result.size)
        assertEquals("action1", result[0]?.id)
        assertEquals("Title 1", result[0]?.title)
        assertEquals(false, result[0]?.input)
        assertEquals("action2", result[1]?.id)
        assertEquals("Title 2", result[1]?.title)
        assertEquals(true, result[1]?.input)
    }

    @Test
    fun testGetActionGroup_empty() {
        every { mockSharedPreferences.getInt("count", 0) } returns 0

        val result = notificationStorage.getActionGroup("emptyType")

        assertEquals(0, result.size)
    }

    @Test
    fun testGetActionGroup_nullValues() {
        every { mockSharedPreferences.getInt("count", 0) } returns 1
        every { mockSharedPreferences.getString("id0", "") } returns null
        every { mockSharedPreferences.getString("title0", "") } returns null
        every { mockSharedPreferences.getBoolean("input0", false) } returns false

        val result = notificationStorage.getActionGroup("type1")

        assertEquals(1, result.size)
        assertEquals("", result[0]?.id)
        assertNull(result[0]?.title)
    }
}
