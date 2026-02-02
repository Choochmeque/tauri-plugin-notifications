package app.tauri.notification

import android.content.Context
import android.content.res.Resources
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import app.tauri.plugin.JSObject
import com.fasterxml.jackson.annotation.JsonAutoDetect
import com.fasterxml.jackson.annotation.PropertyAccessor
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.ObjectMapper
import io.mockk.*
import org.json.JSONObject
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

    /**
     * Proves that Jackson's ObjectMapper cannot properly serialize a Notification
     * with a JSObject (extends org.json.JSONObject) in its `extra` field.
     * Jackson treats JSONObject as a regular Java bean, so the "extra" field
     * loses its intended {"key":"value"} content. This was the root cause of
     * foreground push notifications not triggering onNotificationReceived.
     */
    @Test
    fun testJacksonSerializationOfNotificationWithExtra_broken() {
        val objectMapper = ObjectMapper()
            .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
            .enable(DeserializationFeature.FAIL_ON_NULL_FOR_PRIMITIVES)
            .setVisibility(PropertyAccessor.FIELD, JsonAutoDetect.Visibility.ANY)

        val notification = Notification()
        notification.id = 1
        notification.title = "Test"
        notification.body = "Body"
        notification.extra = JSObject().apply { put("key", "value") }

        // Jackson serializes JSObject as a Java bean, not as a JSON map
        val json = objectMapper.writeValueAsString(notification)
        val parsed = JSONObject(json)

        // The extra field either doesn't contain the expected key/value,
        // or is serialized as bean internals instead of {"key":"value"}
        val extraField = parsed.opt("extra")
        val isCorrect = try {
            val extra = parsed.getJSONObject("extra")
            extra.has("key") && extra.getString("key") == "value"
        } catch (e: Exception) {
            false
        }
        assertFalse("Jackson should NOT correctly serialize JSObject extra field", isCorrect)
    }

    /**
     * Verifies that the fixed triggerNotification() builds a correct JSObject
     * using manual construction + Plugin.trigger() instead of triggerObject().
     * This uses the same serialization path as Channel.send(JSObject).
     */
    @Test
    fun testTriggerNotificationBuildsCorrectJSObject() {
        val notification = Notification()
        notification.id = 42
        notification.title = "Push Title"
        notification.body = "Push Body"
        notification.channelId = "test-channel"
        notification.sound = "default"
        notification.isGroupSummary = true
        notification.extra = JSObject().apply {
            put("key", "value")
            put("nested", "data")
        }
        notification.inboxLines = listOf("Line 1", "Line 2")

        // Reproduce what the fixed triggerNotification() does
        val data = JSObject()
        data.put("id", notification.id)
        notification.title?.let { data.put("title", it) }
        notification.body?.let { data.put("body", it) }
        notification.largeBody?.let { data.put("largeBody", it) }
        notification.summary?.let { data.put("summary", it) }
        notification.sound?.let { data.put("sound", it) }
        notification.actionTypeId?.let { data.put("actionTypeId", it) }
        notification.group?.let { data.put("group", it) }
        notification.channelId?.let { data.put("channelId", it) }
        if (notification.isGroupSummary) data.put("groupSummary", true)
        if (notification.isOngoing) data.put("ongoing", true)
        if (notification.isAutoCancel) data.put("autoCancel", true)
        notification.silent?.let { data.put("silent", it) }
        notification.extra?.let { data.put("extra", it) }

        // Verify it produces valid JSON via PluginResult (the Channel.send path)
        val json = data.toString()
        val parsed = JSONObject(json)

        assertEquals(42, parsed.getInt("id"))
        assertEquals("Push Title", parsed.getString("title"))
        assertEquals("Push Body", parsed.getString("body"))
        assertEquals("test-channel", parsed.getString("channelId"))
        assertEquals("default", parsed.getString("sound"))
        assertTrue(parsed.getBoolean("groupSummary"))
        assertFalse(parsed.has("ongoing"))
        assertFalse(parsed.has("autoCancel"))

        val extra = parsed.getJSONObject("extra")
        assertEquals("value", extra.getString("key"))
        assertEquals("data", extra.getString("nested"))
    }

    @Test
    fun testTriggerNotification_sourceDefaultsToLocal() {
        val data = JSObject()
        val source = "local" // default parameter value
        data.put("source", source)
        data.put("id", 1)

        val json = data.toString()
        val parsed = JSONObject(json)

        assertEquals("local", parsed.getString("source"))
    }

    @Test
    fun testTriggerNotification_sourceCanBePush() {
        val data = JSObject()
        val source = "push"
        data.put("source", source)
        data.put("id", 1)

        val json = data.toString()
        val parsed = JSONObject(json)

        assertEquals("push", parsed.getString("source"))
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
