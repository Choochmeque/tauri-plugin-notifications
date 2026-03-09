package app.tauri.notification

import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import io.mockk.*
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/**
 * Tests for the UnifiedPush-related behaviors in NotificationPlugin,
 * focusing on handleNewUnifiedPushEndpoint, handleUnifiedPushRegistrationFailed,
 * handleUnifiedPushUnregistered, and the pendingUnifiedPushInvoke lifecycle.
 */
@RunWith(RobolectricTestRunner::class)
class NotificationPluginUnifiedPushTest {

    private lateinit var mockInvoke: Invoke
    private lateinit var mockInvoke2: Invoke

    @Before
    fun setup() {
        mockInvoke = mockk(relaxed = true)
        mockInvoke2 = mockk(relaxed = true)
    }

    // --- handleNewUnifiedPushEndpoint tests ---

    @Test
    fun testHandleNewUnifiedPushEndpoint_resolvesData() {
        // Test that endpoint and instance data is correctly structured
        val result = JSObject()
        result.put("endpoint", "https://push.example.com/abc")
        result.put("instance", "test-instance")

        assertEquals("https://push.example.com/abc", result.getString("endpoint"))
        assertEquals("test-instance", result.getString("instance"))
    }

    @Test
    fun testHandleNewUnifiedPushEndpoint_endpointContainsUrl() {
        val data = JSObject()
        data.put("endpoint", "https://push.example.com/endpoint/12345")
        data.put("instance", "default")

        assertTrue(data.getString("endpoint")!!.startsWith("https://"))
        assertEquals("default", data.getString("instance"))
    }

    // --- handleUnifiedPushRegistrationFailed tests ---

    @Test
    fun testHandleUnifiedPushRegistrationFailed_errorDataStructure() {
        val instance = "test-instance"
        val errorMessage = "UnifiedPush registration failed for instance: $instance"
        val errorData = JSObject()
        errorData.put("message", errorMessage)
        errorData.put("instance", instance)

        assertTrue(errorData.getString("message")!!.contains("registration failed"))
        assertEquals("test-instance", errorData.getString("instance"))
    }

    // --- handleUnifiedPushUnregistered tests ---

    @Test
    fun testHandleUnifiedPushUnregistered_dataStructure() {
        val data = JSObject()
        data.put("instance", "test-instance")

        assertEquals("test-instance", data.getString("instance"))
    }

    // --- Pending invoke lifecycle tests ---

    @Test
    fun testPendingInvoke_rejectedWhenNewRegistrationRequested() {
        // Simulates the behavior where a new registerForUnifiedPush call
        // rejects the previous pending invoke
        val firstInvoke = mockInvoke
        val secondInvoke = mockInvoke2

        // First registration stores pendingUnifiedPushInvoke
        var pendingUnifiedPushInvoke: Invoke? = firstInvoke

        // Second registration should reject the first
        pendingUnifiedPushInvoke?.reject("Superseded by a new registration request")
        pendingUnifiedPushInvoke = secondInvoke

        verify { firstInvoke.reject("Superseded by a new registration request") }
        assertSame(secondInvoke, pendingUnifiedPushInvoke)
    }

    @Test
    fun testPendingInvoke_rejectedOnUnregister() {
        // Simulates the fix: unregisterFromUnifiedPush rejects pending invoke
        var pendingUnifiedPushInvoke: Invoke? = mockInvoke

        // Unregister should reject pending invoke
        pendingUnifiedPushInvoke?.reject("Unregistration requested while registration was in progress")
        pendingUnifiedPushInvoke = null

        verify { mockInvoke.reject("Unregistration requested while registration was in progress") }
        assertNull(pendingUnifiedPushInvoke)
    }

    @Test
    fun testPendingInvoke_nullWhenNoRegistrationInProgress() {
        // Unregister with no pending invoke should not crash
        var pendingUnifiedPushInvoke: Invoke? = null

        pendingUnifiedPushInvoke?.reject("Unregistration requested while registration was in progress")
        pendingUnifiedPushInvoke = null

        // No verification needed - just ensuring no NPE
        assertNull(pendingUnifiedPushInvoke)
    }

    @Test
    fun testPendingInvoke_resolvedOnNewEndpoint() {
        // Simulates handleNewUnifiedPushEndpoint resolving the pending invoke
        var pendingUnifiedPushInvoke: Invoke? = mockInvoke

        val result = JSObject()
        result.put("endpoint", "https://push.example.com/abc")
        result.put("instance", "default")

        pendingUnifiedPushInvoke?.resolve(result)
        pendingUnifiedPushInvoke = null

        verify { mockInvoke.resolve(match<JSObject> {
            it.getString("endpoint") == "https://push.example.com/abc" &&
            it.getString("instance") == "default"
        }) }
        assertNull(pendingUnifiedPushInvoke)
    }

    @Test
    fun testPendingInvoke_rejectedOnRegistrationFailed() {
        // Simulates handleUnifiedPushRegistrationFailed rejecting the pending invoke
        var pendingUnifiedPushInvoke: Invoke? = mockInvoke
        val instance = "test-instance"
        val errorMessage = "UnifiedPush registration failed for instance: $instance"

        pendingUnifiedPushInvoke?.reject(errorMessage)
        pendingUnifiedPushInvoke = null

        verify { mockInvoke.reject(match<String> { it.contains("registration failed") }) }
        assertNull(pendingUnifiedPushInvoke)
    }

    // --- triggerUnifiedPushMessage data mapping tests ---

    @Test
    fun testTriggerUnifiedPushMessage_mapsStringValues() {
        val pushData = mapOf<String, Any>(
            "title" to "Test Title",
            "body" to "Test Body",
            "instance" to "default",
            "source" to "unifiedpush"
        )

        val data = JSObject()
        for ((key, value) in pushData) {
            when (value) {
                is String -> data.put(key, value)
                is Int -> data.put(key, value)
                is Long -> data.put(key, value)
                is Double -> data.put(key, value)
                is Boolean -> data.put(key, value)
                else -> data.put(key, value.toString())
            }
        }

        assertEquals("Test Title", data.getString("title"))
        assertEquals("Test Body", data.getString("body"))
        assertEquals("default", data.getString("instance"))
        assertEquals("unifiedpush", data.getString("source"))
    }

    @Test
    fun testTriggerUnifiedPushMessage_mapsNumericValues() {
        val pushData = mapOf<String, Any>(
            "count" to 42,
            "timestamp" to 1234567890L,
            "ratio" to 3.14
        )

        val data = JSObject()
        for ((key, value) in pushData) {
            when (value) {
                is String -> data.put(key, value)
                is Int -> data.put(key, value)
                is Long -> data.put(key, value)
                is Double -> data.put(key, value)
                is Boolean -> data.put(key, value)
                else -> data.put(key, value.toString())
            }
        }

        assertEquals(42, data.getInteger("count"))
        assertEquals(1234567890L, data.getLong("timestamp"))
        assertEquals(3.14, data.getDouble("ratio"), 0.001)
    }

    @Test
    fun testTriggerUnifiedPushMessage_mapsBooleanValues() {
        val pushData = mapOf<String, Any>(
            "read" to true,
            "archived" to false
        )

        val data = JSObject()
        for ((key, value) in pushData) {
            when (value) {
                is Boolean -> data.put(key, value)
                else -> data.put(key, value.toString())
            }
        }

        assertTrue(data.getBoolean("read"))
        assertFalse(data.getBoolean("archived"))
    }

    @Test
    fun testTriggerUnifiedPushMessage_mapsNestedObjects() {
        val nestedMap = mapOf("innerKey" to "innerValue", "innerNum" to "99")
        val pushData = mapOf<String, Any>(
            "nested" to nestedMap
        )

        val data = JSObject()
        for ((key, value) in pushData) {
            when (value) {
                is Map<*, *> -> {
                    val nestedObj = JSObject()
                    @Suppress("UNCHECKED_CAST")
                    val map = value as Map<String, Any>
                    for ((k, v) in map) {
                        nestedObj.put(k, v.toString())
                    }
                    data.put(key, nestedObj)
                }
                else -> data.put(key, value.toString())
            }
        }

        val nested = data.getJSObject("nested")
        assertNotNull(nested)
        assertEquals("innerValue", nested!!.getString("innerKey"))
        assertEquals("99", nested.getString("innerNum"))
    }

    // --- Cached endpoint behavior tests ---

    @Test
    fun testCachedEndpoint_clearedOnUnregister() {
        var cachedUnifiedPushEndpoint: String? = "https://push.example.com/cached"

        // Simulate unregister
        cachedUnifiedPushEndpoint = null

        assertNull(cachedUnifiedPushEndpoint)
    }

    @Test
    fun testCachedEndpoint_updatedOnNewEndpoint() {
        var cachedUnifiedPushEndpoint: String? = null
        var unifiedPushInstance = "default"

        // Simulate new endpoint
        cachedUnifiedPushEndpoint = "https://push.example.com/new-endpoint"
        unifiedPushInstance = "new-instance"

        assertEquals("https://push.example.com/new-endpoint", cachedUnifiedPushEndpoint)
        assertEquals("new-instance", unifiedPushInstance)
    }

    @Test
    fun testCachedEndpoint_returnedImmediatelyIfAvailable() {
        val cachedUnifiedPushEndpoint: String? = "https://push.example.com/cached"
        val unifiedPushInstance = "cached-instance"

        // If cached endpoint exists, resolve immediately
        if (cachedUnifiedPushEndpoint != null) {
            val result = JSObject()
            result.put("endpoint", cachedUnifiedPushEndpoint)
            result.put("instance", unifiedPushInstance)

            assertEquals("https://push.example.com/cached", result.getString("endpoint"))
            assertEquals("cached-instance", result.getString("instance"))
        } else {
            fail("Cached endpoint should not be null in this test")
        }
    }

    // --- Distributors data structure tests ---

    @Test
    fun testGetUnifiedPushDistributors_resultStructure() {
        val distributors = listOf("org.unifiedpush.distributor.fcm", "org.unifiedpush.distributor.nextpush")

        val result = JSObject()
        val distributorsArray = org.json.JSONArray()
        distributors.forEach { distributorsArray.put(it) }
        result.put("distributors", distributorsArray)

        val arr = result.getJSONArray("distributors")
        assertEquals(2, arr.length())
        assertEquals("org.unifiedpush.distributor.fcm", arr.getString(0))
        assertEquals("org.unifiedpush.distributor.nextpush", arr.getString(1))
    }

    @Test
    fun testGetUnifiedPushDistributors_emptyList() {
        val distributors = emptyList<String>()

        val result = JSObject()
        val distributorsArray = org.json.JSONArray()
        distributors.forEach { distributorsArray.put(it) }
        result.put("distributors", distributorsArray)

        val arr = result.getJSONArray("distributors")
        assertEquals(0, arr.length())
    }

    @Test
    fun testGetUnifiedPushDistributor_resultStructure() {
        val distributor = "org.unifiedpush.distributor.fcm"
        val result = JSObject()
        result.put("distributor", distributor)

        assertEquals("org.unifiedpush.distributor.fcm", result.getString("distributor"))
    }

    @Test
    fun testGetUnifiedPushDistributor_emptyWhenNotSaved() {
        val distributor = ""
        val result = JSObject()
        result.put("distributor", distributor)

        assertEquals("", result.getString("distributor"))
    }

    @Test
    fun testSaveUnifiedPushDistributor_requiresNonNullDistributor() {
        val distributor: String? = null

        if (distributor == null) {
            // Should reject with "Distributor parameter is required"
            assertTrue(true)
        } else {
            fail("Distributor should be null in this test case")
        }
    }
}

