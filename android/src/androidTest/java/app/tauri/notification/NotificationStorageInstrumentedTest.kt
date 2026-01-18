package app.tauri.notification

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.fasterxml.jackson.databind.ObjectMapper
import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class NotificationStorageInstrumentedTest {

    private lateinit var storage: NotificationStorage
    private lateinit var objectMapper: ObjectMapper

    @Before
    fun setup() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        objectMapper = ObjectMapper()
        storage = NotificationStorage(context, objectMapper)

        // Clear any existing data
        clearAllNotifications()
    }

    @After
    fun cleanup() {
        clearAllNotifications()
    }

    private fun clearAllNotifications() {
        val ids = storage.getSavedNotificationIds()
        for (id in ids) {
            storage.deleteNotification(id)
        }
    }

    @Test
    fun testSaveAndRetrieveNotification() {
        val notification = Notification()
        notification.id = 100
        notification.title = "Test Notification"
        notification.body = "Test Body"
        notification.schedule = NotificationSchedule.At().apply {
            repeating = false
        }
        notification.sourceJson = """{"id":100,"title":"Test Notification","body":"Test Body"}"""

        storage.appendNotifications(listOf(notification))

        val ids = storage.getSavedNotificationIds()
        assertTrue(ids.contains("100"))

        val retrieved = storage.getSavedNotification("100")
        assertNotNull(retrieved)
        assertEquals(100, retrieved?.id)
        assertEquals("Test Notification", retrieved?.title)
    }

    @Test
    fun testSaveMultipleNotifications() {
        val notifications = listOf(
            Notification().apply {
                id = 1
                title = "First"
                schedule = NotificationSchedule.At()
                sourceJson = """{"id":1}"""
            },
            Notification().apply {
                id = 2
                title = "Second"
                schedule = NotificationSchedule.Every().apply {
                    interval = NotificationInterval.Hour
                    count = 1
                }
                sourceJson = """{"id":2}"""
            },
            Notification().apply {
                id = 3
                title = "Third"
                schedule = NotificationSchedule.Interval().apply {
                    interval = DateMatch()
                }
                sourceJson = """{"id":3}"""
            }
        )

        storage.appendNotifications(notifications)

        val ids = storage.getSavedNotificationIds()
        assertEquals(3, ids.size)
        assertTrue(ids.contains("1"))
        assertTrue(ids.contains("2"))
        assertTrue(ids.contains("3"))
    }

    @Test
    fun testDeleteNotification() {
        val notification = Notification()
        notification.id = 200
        notification.title = "To Delete"
        notification.schedule = NotificationSchedule.At()
        notification.sourceJson = """{"id":200}"""

        storage.appendNotifications(listOf(notification))

        var ids = storage.getSavedNotificationIds()
        assertTrue(ids.contains("200"))

        storage.deleteNotification("200")

        ids = storage.getSavedNotificationIds()
        assertFalse(ids.contains("200"))
    }

    @Test
    fun testGetSavedNotifications() {
        val notifications = listOf(
            Notification().apply {
                id = 10
                title = "First"
                body = "Body 1"
                schedule = NotificationSchedule.At()
                sourceJson = """{"id":10,"title":"First","body":"Body 1"}"""
            },
            Notification().apply {
                id = 20
                title = "Second"
                body = "Body 2"
                schedule = NotificationSchedule.At()
                sourceJson = """{"id":20,"title":"Second","body":"Body 2"}"""
            }
        )

        storage.appendNotifications(notifications)

        val retrieved = storage.getSavedNotifications()
        assertTrue(retrieved.size >= 2)

        val ids = retrieved.map { it.id }
        assertTrue(ids.contains(10))
        assertTrue(ids.contains(20))
    }

    @Test
    fun testSaveActionGroups() {
        val action1 = NotificationAction()
        action1.id = "reply"
        action1.title = "Reply"
        action1.input = true

        val action2 = NotificationAction()
        action2.id = "dismiss"
        action2.title = "Dismiss"
        action2.input = false

        val actionType = ActionType()
        actionType.id = "message-actions"
        actionType.actions = listOf(action1, action2)

        storage.writeActionGroup(listOf(actionType))

        val retrieved = storage.getActionGroup("message-actions")

        assertEquals(2, retrieved.size)
        assertEquals("reply", retrieved[0]?.id)
        assertEquals("Reply", retrieved[0]?.title)
        assertEquals(true, retrieved[0]?.input)
        assertEquals("dismiss", retrieved[1]?.id)
        assertEquals("Dismiss", retrieved[1]?.title)
        assertEquals(false, retrieved[1]?.input)
    }

    @Test
    fun testOverwriteNotification() {
        val notification1 = Notification()
        notification1.id = 300
        notification1.title = "Original"
        notification1.schedule = NotificationSchedule.At()
        notification1.sourceJson = """{"id":300,"title":"Original"}"""

        storage.appendNotifications(listOf(notification1))

        val notification2 = Notification()
        notification2.id = 300
        notification2.title = "Updated"
        notification2.schedule = NotificationSchedule.At()
        notification2.sourceJson = """{"id":300,"title":"Updated"}"""

        storage.appendNotifications(listOf(notification2))

        val retrieved = storage.getSavedNotification("300")
        assertEquals("Updated", retrieved?.title)
    }

    @Test
    fun testNotificationWithoutScheduleNotSaved() {
        val notification = Notification()
        notification.id = 400
        notification.title = "No Schedule"
        notification.schedule = null
        notification.sourceJson = """{"id":400}"""

        storage.appendNotifications(listOf(notification))

        val ids = storage.getSavedNotificationIds()
        assertFalse(ids.contains("400"))
    }

    @Test
    fun testGetNonExistentNotification() {
        val retrieved = storage.getSavedNotification("999999")
        assertNull(retrieved)
    }

    @Test
    fun testEmptyActionGroup() {
        val retrieved = storage.getActionGroup("nonexistent-group")
        assertEquals(0, retrieved.size)
    }

    @Test
    fun testMultipleActionGroups() {
        val group1Actions = listOf(
            NotificationAction().apply {
                id = "action1"
                title = "Action 1"
                input = false
            }
        )
        val group1 = ActionType().apply {
            id = "group1"
            actions = group1Actions
        }

        val group2Actions = listOf(
            NotificationAction().apply {
                id = "action2"
                title = "Action 2"
                input = true
            },
            NotificationAction().apply {
                id = "action3"
                title = "Action 3"
                input = false
            }
        )
        val group2 = ActionType().apply {
            id = "group2"
            actions = group2Actions
        }

        storage.writeActionGroup(listOf(group1, group2))

        val retrieved1 = storage.getActionGroup("group1")
        assertEquals(1, retrieved1.size)
        assertEquals("action1", retrieved1[0]?.id)
        assertEquals("Action 1", retrieved1[0]?.title)
        assertEquals(false, retrieved1[0]?.input)

        val retrieved2 = storage.getActionGroup("group2")
        assertEquals(2, retrieved2.size)
        assertEquals("action2", retrieved2[0]?.id)
        assertEquals("Action 2", retrieved2[0]?.title)
        assertEquals(true, retrieved2[0]?.input)
        assertEquals("action3", retrieved2[1]?.id)
        assertEquals("Action 3", retrieved2[1]?.title)
        assertEquals(false, retrieved2[1]?.input)
    }
}
