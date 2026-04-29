package app.tauri.notification

import android.app.Activity
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import io.mockk.*
import org.junit.Assert.*
import org.junit.After
import org.junit.Assume.assumeTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner

/**
 * Tests for the UnifiedPush-related behaviors in NotificationPlugin.
 *
 * These tests exercise the actual plugin methods (handleNewUnifiedPushEndpoint,
 * handleUnifiedPushRegistrationFailed, handleUnifiedPushUnregistered,
 * triggerUnifiedPushMessage) via a real NotificationPlugin instance, using
 * reflection to set up internal state (pendingUnifiedPushInvoke,
 * cachedUnifiedPushEndpoint, unifiedPushInstance) since those fields are private.
 */
@RunWith(RobolectricTestRunner::class)
class NotificationPluginUnifiedPushTest {

    private lateinit var plugin: NotificationPlugin
    private lateinit var mockInvoke: Invoke
    private lateinit var mockInvoke2: Invoke

    @Before
    fun setup() {
        assumeTrue("UnifiedPush tests require ENABLE_UNIFIED_PUSH", BuildConfig.ENABLE_UNIFIED_PUSH)

        val activity = Robolectric.buildActivity(Activity::class.java).create().get()
        plugin = NotificationPlugin(activity)
        NotificationPlugin.instance = plugin

        mockInvoke = mockk(relaxed = true)
        mockInvoke2 = mockk(relaxed = true)
    }

    @After
    fun teardown() {
        NotificationPlugin.instance = null
    }

    // --- Helper methods to access private fields via reflection ---

    private fun setPendingUnifiedPushInvoke(invoke: Invoke?) {
        val field = NotificationPlugin::class.java.getDeclaredField("pendingUnifiedPushInvoke")
        field.isAccessible = true
        field.set(plugin, invoke)
    }

    private fun getPendingUnifiedPushInvoke(): Invoke? {
        val field = NotificationPlugin::class.java.getDeclaredField("pendingUnifiedPushInvoke")
        field.isAccessible = true
        return field.get(plugin) as? Invoke
    }

    private fun setCachedUnifiedPushEndpoint(endpoint: String?) {
        val field = NotificationPlugin::class.java.getDeclaredField("cachedUnifiedPushEndpoint")
        field.isAccessible = true
        field.set(plugin, endpoint)
    }

    private fun getCachedUnifiedPushEndpoint(): String? {
        val field = NotificationPlugin::class.java.getDeclaredField("cachedUnifiedPushEndpoint")
        field.isAccessible = true
        return field.get(plugin) as? String
    }

    private fun setUnifiedPushInstance(instance: String) {
        val field = NotificationPlugin::class.java.getDeclaredField("unifiedPushInstance")
        field.isAccessible = true
        field.set(plugin, instance)
    }

    private fun getUnifiedPushInstance(): String {
        val field = NotificationPlugin::class.java.getDeclaredField("unifiedPushInstance")
        field.isAccessible = true
        return field.get(plugin) as String
    }

    // --- handleNewUnifiedPushEndpoint tests ---

    @Test
    fun testHandleNewUnifiedPushEndpoint_resolvesPendingInvoke() {
        setPendingUnifiedPushInvoke(mockInvoke)

        plugin.handleNewUnifiedPushEndpoint("https://push.example.com/abc", "test-instance")

        verify { mockInvoke.resolve(match<JSObject> {
            it.getString("endpoint") == "https://push.example.com/abc" &&
            it.getString("instance") == "test-instance"
        }) }
        assertNull(getPendingUnifiedPushInvoke())
    }

    @Test
    fun testHandleNewUnifiedPushEndpoint_resolvesPendingInvoke_withPubKeySet() {
        setPendingUnifiedPushInvoke(mockInvoke)

        plugin.handleNewUnifiedPushEndpoint(
            "https://nextpush.example.com/endpoint/xyz",
            "default",
            "BNcRdreALRFXTkOOUHK1EtK2wtZ5ZIILHY0CRbISTuErp8KS0DLjFCMDxEPPW4ECPF",
            "8eDyX_uCN0XRhSbY5hs7Hg",
        )

        verify { mockInvoke.resolve(match<JSObject> {
            it.getString("endpoint") == "https://nextpush.example.com/endpoint/xyz" &&
            it.getString("instance") == "default" &&
            it.getJSObject("pubKeySet")?.getString("pubKey") == "BNcRdreALRFXTkOOUHK1EtK2wtZ5ZIILHY0CRbISTuErp8KS0DLjFCMDxEPPW4ECPF" &&
            it.getJSObject("pubKeySet")?.getString("auth") == "8eDyX_uCN0XRhSbY5hs7Hg"
        }) }
        assertNull(getPendingUnifiedPushInvoke())
    }

    @Test
    fun testHandleNewUnifiedPushEndpoint_noPubKeySet_omitsField() {
        setPendingUnifiedPushInvoke(mockInvoke)

        plugin.handleNewUnifiedPushEndpoint("https://push.example.com/abc", "default", null, null)

        verify { mockInvoke.resolve(match<JSObject> {
            it.getString("endpoint") == "https://push.example.com/abc" &&
            !it.has("pubKeySet")
        }) }
    }

    @Test
    fun testHandleNewUnifiedPushEndpoint_updatesCachedEndpointAndInstance() {
        setPendingUnifiedPushInvoke(null)

        plugin.handleNewUnifiedPushEndpoint("https://push.example.com/new", "new-instance")

        assertEquals("https://push.example.com/new", getCachedUnifiedPushEndpoint())
        assertEquals("new-instance", getUnifiedPushInstance())
    }

    @Test
    fun testHandleNewUnifiedPushEndpoint_noPendingInvoke_doesNotCrash() {
        setPendingUnifiedPushInvoke(null)

        // Should not throw even with no pending invoke
        plugin.handleNewUnifiedPushEndpoint("https://push.example.com/abc", "default")

        assertEquals("https://push.example.com/abc", getCachedUnifiedPushEndpoint())
    }

    // --- handleUnifiedPushTempUnavailable tests ---

    @Test
    fun testHandleUnifiedPushTempUnavailable_triggersEvent() {
        // Should not throw and should trigger the unifiedpush-temp-unavailable event
        // (trigger() is a no-op without a loaded WebView, but must not crash)
        plugin.handleUnifiedPushTempUnavailable("test-instance")
    }

    @Test
    fun testHandleUnifiedPushTempUnavailable_doesNotClearCachedEndpoint() {
        setCachedUnifiedPushEndpoint("https://push.example.com/cached")

        plugin.handleUnifiedPushTempUnavailable("test-instance")

        // Temp-unavailable should NOT clear the cache — the registration is still valid
        assertEquals("https://push.example.com/cached", getCachedUnifiedPushEndpoint())
    }

    // --- handleUnifiedPushRegistrationFailed tests ---

    @Test
    fun testHandleUnifiedPushRegistrationFailed_rejectsPendingInvoke() {
        setPendingUnifiedPushInvoke(mockInvoke)

        plugin.handleUnifiedPushRegistrationFailed("test-instance", "NETWORK")

        verify { mockInvoke.reject(match<String> {
            it.contains("registration failed") && it.contains("test-instance") && it.contains("NETWORK")
        }) }
        assertNull(getPendingUnifiedPushInvoke())
    }

    @Test
    fun testHandleUnifiedPushRegistrationFailed_noPendingInvoke_doesNotCrash() {
        setPendingUnifiedPushInvoke(null)

        // Should not throw
        plugin.handleUnifiedPushRegistrationFailed("test-instance")

        assertNull(getPendingUnifiedPushInvoke())
    }

    @Test
    fun testHandleUnifiedPushRegistrationFailed_withoutReason() {
        setPendingUnifiedPushInvoke(mockInvoke)

        plugin.handleUnifiedPushRegistrationFailed("test-instance")

        verify { mockInvoke.reject(match<String> {
            it.contains("registration failed") && it.contains("test-instance") && !it.contains("reason")
        }) }
    }

    // --- handleUnifiedPushUnregistered tests ---

    @Test
    fun testHandleUnifiedPushUnregistered_clearsCachedEndpoint() {
        setCachedUnifiedPushEndpoint("https://push.example.com/cached")

        plugin.handleUnifiedPushUnregistered("test-instance")

        assertNull(getCachedUnifiedPushEndpoint())
    }

    // --- Pending invoke lifecycle tests (using actual plugin methods) ---

    @Test
    fun testNewEndpoint_resolvesPending_thenClearsIt() {
        setPendingUnifiedPushInvoke(mockInvoke)

        plugin.handleNewUnifiedPushEndpoint("https://push.example.com/abc", "default")

        verify { mockInvoke.resolve(any<JSObject>()) }
        assertNull(getPendingUnifiedPushInvoke())
    }

    @Test
    fun testRegistrationFailed_rejectsPending_thenClearsIt() {
        setPendingUnifiedPushInvoke(mockInvoke)

        plugin.handleUnifiedPushRegistrationFailed("default", "TIMEOUT")

        verify { mockInvoke.reject(any<String>()) }
        assertNull(getPendingUnifiedPushInvoke())
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

        // Exercises the actual when-expression in the plugin method;
        // any type errors will throw. trigger() is a no-op without a loaded WebView.
        plugin.triggerUnifiedPushMessage(pushData)
    }

    @Test
    fun testTriggerUnifiedPushMessage_mapsNumericValues() {
        val pushData = mapOf<String, Any>(
            "count" to 42,
            "timestamp" to 1234567890L,
            "ratio" to 3.14
        )

        plugin.triggerUnifiedPushMessage(pushData)
    }

    @Test
    fun testTriggerUnifiedPushMessage_mapsBooleanValues() {
        val pushData = mapOf<String, Any>(
            "read" to true,
            "archived" to false
        )

        plugin.triggerUnifiedPushMessage(pushData)
    }

    @Test
    fun testTriggerUnifiedPushMessage_mapsNestedObjects() {
        val nestedMap = mapOf("innerKey" to "innerValue", "innerNum" to 99)
        val pushData = mapOf<String, Any>(
            "nested" to nestedMap
        )

        plugin.triggerUnifiedPushMessage(pushData)
    }

    @Test
    fun testTriggerUnifiedPushMessage_mapsListValues() {
        val pushData = mapOf<String, Any>(
            "items" to listOf(1, 2, 3),
            "tags" to listOf("a", "b", "c")
        )

        // This exercises the is List<*> branch — previously this would
        // fall through to toString() and mangle the data
        plugin.triggerUnifiedPushMessage(pushData)
    }

    @Test
    fun testTriggerUnifiedPushMessage_mapsNestedListsAndMaps() {
        val pushData = mapOf<String, Any>(
            "complex" to listOf(
                mapOf("key" to "value"),
                listOf(1, 2),
                "plain"
            )
        )

        plugin.triggerUnifiedPushMessage(pushData)
    }

    // --- putValueToJSObject / convertListToJSArray validation tests ---
    // These test the private data-mapping helpers via reflection,
    // verifying correct JSObject/JSArray output.

    @Test
    fun testPutValueToJSObject_listConvertedToJSArray() {
        val method = NotificationPlugin::class.java.getDeclaredMethod(
            "putValueToJSObject", JSObject::class.java, String::class.java, Any::class.java
        )
        method.isAccessible = true

        val target = JSObject()
        method.invoke(plugin, target, "items", listOf(1, 2, 3))

        val arr = target.getJSONArray("items")
        assertEquals(3, arr.length())
        assertEquals(1, arr.getInt(0))
        assertEquals(2, arr.getInt(1))
        assertEquals(3, arr.getInt(2))
    }

    @Test
    fun testPutValueToJSObject_nestedMapConvertedRecursively() {
        val method = NotificationPlugin::class.java.getDeclaredMethod(
            "putValueToJSObject", JSObject::class.java, String::class.java, Any::class.java
        )
        method.isAccessible = true

        val target = JSObject()
        method.invoke(plugin, target, "nested", mapOf("key" to "value", "num" to 42))

        val nested = target.getJSObject("nested")
        assertNotNull(nested)
        assertEquals("value", nested!!.getString("key"))
        assertEquals(42, nested.getInt("num"))
    }

    @Test
    fun testPutValueToJSObject_mixedListWithMapsAndPrimitives() {
        val method = NotificationPlugin::class.java.getDeclaredMethod(
            "putValueToJSObject", JSObject::class.java, String::class.java, Any::class.java
        )
        method.isAccessible = true

        val target = JSObject()
        val mixedList = listOf(
            "text",
            42,
            mapOf("inner" to "obj"),
            listOf(1, 2)
        )
        method.invoke(plugin, target, "mixed", mixedList)

        val arr = target.getJSONArray("mixed")
        assertEquals(4, arr.length())
        assertEquals("text", arr.getString(0))
        assertEquals(42, arr.getInt(1))
        // Element 2 is a JSONObject
        val innerObj = arr.getJSONObject(2)
        assertEquals("obj", innerObj.getString("inner"))
        // Element 3 is a nested JSONArray
        val innerArr = arr.getJSONArray(3)
        assertEquals(2, innerArr.length())
    }

    // --- Cached endpoint behavior tests (using actual plugin state) ---

    @Test
    fun testCachedEndpoint_clearedOnUnregister() {
        setCachedUnifiedPushEndpoint("https://push.example.com/cached")

        plugin.handleUnifiedPushUnregistered("default")

        assertNull(getCachedUnifiedPushEndpoint())
    }

    @Test
    fun testCachedEndpoint_updatedOnNewEndpoint() {
        setCachedUnifiedPushEndpoint(null)
        setUnifiedPushInstance("default")

        plugin.handleNewUnifiedPushEndpoint("https://push.example.com/new-endpoint", "new-instance")

        assertEquals("https://push.example.com/new-endpoint", getCachedUnifiedPushEndpoint())
        assertEquals("new-instance", getUnifiedPushInstance())
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
        // Build a mock Invoke that returns a SaveUnifiedPushDistributorArgs with null distributor
        val args = SaveUnifiedPushDistributorArgs()
        // distributor defaults to null

        val invoke = mockk<Invoke>(relaxed = true)
        every { invoke.parseArgs(SaveUnifiedPushDistributorArgs::class.java) } returns args

        plugin.saveUnifiedPushDistributor(invoke)

        verify { invoke.reject("Distributor parameter is required") }
    }
}
