package app.tauri.notification

import android.content.Context
import app.tauri.plugin.JSObject
import io.mockk.*
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.unifiedpush.android.connector.FailedReason
import org.unifiedpush.android.connector.data.PushEndpoint
import org.unifiedpush.android.connector.data.PushMessage
import java.util.concurrent.Executor

@RunWith(RobolectricTestRunner::class)
class TauriUnifiedPushMessagingServiceTest {

    private lateinit var service: TauriUnifiedPushMessagingService
    private lateinit var mockContext: Context
    private lateinit var mockPlugin: NotificationPlugin

    @Before
    fun setup() {
        service = TauriUnifiedPushMessagingService()
        mockContext = mockk(relaxed = true)
        mockPlugin = mockk(relaxed = true)
        NotificationPlugin.instance = mockPlugin

        // Clear any custom message handler between tests
        TauriUnifiedPushMessagingService.setMessageHandler(null)

        // Use a synchronous executor so handler tests don't need Thread.sleep
        TauriUnifiedPushMessagingService.setExecutorForTesting(Executor { it.run() })
    }

    // --- onMessage JSON parsing tests ---

    @Test
    fun testOnMessage_validJsonParsedCorrectly() {
        val json = """{"title":"Test Title","body":"Test Body","extra_key":"extra_value"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        val capturedData = slot<Map<String, Any>>()
        every { mockPlugin.triggerUnifiedPushMessage(capture(capturedData)) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "test-instance")

        assertTrue(capturedData.isCaptured)
        val data = capturedData.captured
        assertEquals("Test Title", data["title"])
        assertEquals("Test Body", data["body"])
        assertEquals("extra_value", data["extra_key"])
        assertEquals("test-instance", data["instance"])
        assertEquals("unifiedpush", data["source"])
    }

    @Test
    fun testOnMessage_invalidJsonForwardedAsRawText() {
        val rawText = "This is not JSON"
        val message = mockk<PushMessage>()
        every { message.content } returns rawText.toByteArray(Charsets.UTF_8)

        val capturedData = slot<Map<String, Any>>()
        every { mockPlugin.triggerUnifiedPushMessage(capture(capturedData)) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "test-instance")

        assertTrue(capturedData.isCaptured)
        val data = capturedData.captured
        assertEquals(rawText, data["body"])
        assertEquals("test-instance", data["instance"])
        assertEquals("unifiedpush", data["source"])
    }

    @Test
    fun testOnMessage_nestedJsonObjectsParsedCorrectly() {
        val json = """{"title":"Nested","data":{"key":"value","num":42}}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        val capturedData = slot<Map<String, Any>>()
        every { mockPlugin.triggerUnifiedPushMessage(capture(capturedData)) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")

        assertTrue(capturedData.isCaptured)
        val data = capturedData.captured
        assertEquals("Nested", data["title"])
        // Nested JSON objects are recursively converted to maps
        @Suppress("UNCHECKED_CAST")
        val nestedData = data["data"] as Map<String, Any>
        assertEquals("value", nestedData["key"])
        assertEquals(42, nestedData["num"])
    }

    @Test
    fun testOnMessage_jsonArrayParsedCorrectly() {
        val json = """{"title":"Array","items":[1,2,3]}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        val capturedData = slot<Map<String, Any>>()
        every { mockPlugin.triggerUnifiedPushMessage(capture(capturedData)) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")

        assertTrue(capturedData.isCaptured)
        val data = capturedData.captured
        @Suppress("UNCHECKED_CAST")
        val items = data["items"] as List<Any>
        assertEquals(3, items.size)
        assertEquals(1, items[0])
        assertEquals(2, items[1])
        assertEquals(3, items[2])
    }

    @Test
    fun testOnMessage_emptyJsonObject() {
        val json = """{}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        val capturedData = slot<Map<String, Any>>()
        every { mockPlugin.triggerUnifiedPushMessage(capture(capturedData)) } just Runs

        service.onMessage(mockContext, message, "default")

        assertTrue(capturedData.isCaptured)
        val data = capturedData.captured
        // Only instance and source should be present
        assertEquals("default", data["instance"])
        assertEquals("unifiedpush", data["source"])
        assertNull(data["title"])
        assertNull(data["body"])
    }

    // --- Fallback notification tests ---

    @Test
    fun testOnMessage_fallbackNotificationShownWhenNoHandler() {
        val json = """{"title":"Fallback Title","body":"Fallback Body"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        every { mockPlugin.triggerUnifiedPushMessage(any()) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")

        // Verify that schedule was called on the manager (fallback notification)
        verify { mockManager.schedule(match<Notification> {
            it.title == "Fallback Title" && it.body == "Fallback Body"
        }) }
    }

    @Test
    fun testOnMessage_noFallbackWhenNoTitleAndNoBody() {
        val json = """{"extra_key":"extra_value"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        every { mockPlugin.triggerUnifiedPushMessage(any()) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")

        // Verify that schedule was NOT called (no title or body)
        verify(exactly = 0) { mockManager.schedule(any<Notification>()) }
    }

    @Test
    fun testOnMessage_fallbackNotificationWithBodyOnly() {
        val json = """{"body":"Body Only"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        every { mockPlugin.triggerUnifiedPushMessage(any()) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")

        verify { mockManager.schedule(match<Notification> {
            it.title == "" && it.body == "Body Only"
        }) }
    }

    // --- Custom message handler tests ---

    @Test
    fun testOnMessage_customHandlerSuppressesFallback() {
        val handler = mockk<UnifiedPushMessageHandler>()
        every { handler.onMessage(any(), any(), any()) } returns true
        TauriUnifiedPushMessagingService.setMessageHandler(handler)

        val json = """{"title":"Custom","body":"Handled"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        every { mockPlugin.triggerUnifiedPushMessage(any()) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")


        verify { handler.onMessage(mockContext, any(), "default") }
        // Fallback should NOT be called since handler returned true
        verify(exactly = 0) { mockManager.schedule(any<Notification>()) }
    }

    @Test
    fun testOnMessage_customHandlerReturnsFalseShowsFallback() {
        val handler = mockk<UnifiedPushMessageHandler>()
        every { handler.onMessage(any(), any(), any()) } returns false
        TauriUnifiedPushMessagingService.setMessageHandler(handler)

        val json = """{"title":"Not Handled","body":"Show Fallback"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        every { mockPlugin.triggerUnifiedPushMessage(any()) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")


        verify { handler.onMessage(mockContext, any(), "default") }
        // Fallback SHOULD be called since handler returned false
        verify { mockManager.schedule(match<Notification> {
            it.title == "Not Handled" && it.body == "Show Fallback"
        }) }
    }

    @Test
    fun testOnMessage_customHandlerExceptionShowsFallback() {
        val handler = mockk<UnifiedPushMessageHandler>()
        every { handler.onMessage(any(), any(), any()) } throws RuntimeException("Handler error")
        TauriUnifiedPushMessagingService.setMessageHandler(handler)

        val json = """{"title":"Error","body":"Fallback on error"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        every { mockPlugin.triggerUnifiedPushMessage(any()) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")


        // Fallback SHOULD be called since handler threw exception
        verify { mockManager.schedule(match<Notification> {
            it.title == "Error" && it.body == "Fallback on error"
        }) }
    }

    // --- onNewEndpoint tests ---

    @Test
    fun testOnNewEndpoint_forwardsToPlugin() {
        val endpoint = mockk<PushEndpoint>()
        every { endpoint.url } returns "https://push.example.com/endpoint/abc123"

        service.onNewEndpoint(mockContext, endpoint, "test-instance")

        verify { mockPlugin.handleNewUnifiedPushEndpoint("https://push.example.com/endpoint/abc123", "test-instance") }
    }

    // --- onRegistrationFailed tests ---

    @Test
    fun testOnRegistrationFailed_forwardsToPlugin() {
        service.onRegistrationFailed(mockContext, FailedReason.NETWORK, "test-instance")

        verify { mockPlugin.handleUnifiedPushRegistrationFailed("test-instance", FailedReason.NETWORK.toString()) }
    }

    // --- onUnregistered tests ---

    @Test
    fun testOnUnregistered_forwardsToPlugin() {
        service.onUnregistered(mockContext, "test-instance")

        verify { mockPlugin.handleUnifiedPushUnregistered("test-instance") }
    }

    // --- setMessageHandler tests ---

    @Test
    fun testSetMessageHandler_canBeSetToNull() {
        val handler = mockk<UnifiedPushMessageHandler>()
        TauriUnifiedPushMessagingService.setMessageHandler(handler)
        TauriUnifiedPushMessagingService.setMessageHandler(null)

        // After setting to null, fallback notification path should be taken
        val json = """{"title":"Test","body":"After null handler"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        every { mockPlugin.triggerUnifiedPushMessage(any()) } just Runs

        val mockManager = mockk<TauriNotificationManager>(relaxed = true)
        every { mockPlugin.getNotificationManager() } returns mockManager

        service.onMessage(mockContext, message, "default")

        // handler should not be called
        verify(exactly = 0) { handler.onMessage(any(), any(), any()) }
        // fallback should be shown
        verify { mockManager.schedule(any<Notification>()) }
    }

    // --- Plugin not initialized tests ---

    @Test
    fun testOnMessage_pluginNotInitialized_doesNotCrash() {
        NotificationPlugin.instance = null

        val json = """{"title":"No Plugin","body":"Should not crash"}"""
        val message = mockk<PushMessage>()
        every { message.content } returns json.toByteArray(Charsets.UTF_8)

        // Should not throw
        service.onMessage(mockContext, message, "default")
    }

    @Test
    fun testOnNewEndpoint_pluginNotInitialized_doesNotCrash() {
        NotificationPlugin.instance = null

        val endpoint = mockk<PushEndpoint>()
        every { endpoint.url } returns "https://push.example.com/endpoint/abc123"

        // Should not throw
        service.onNewEndpoint(mockContext, endpoint, "test-instance")
    }
}

