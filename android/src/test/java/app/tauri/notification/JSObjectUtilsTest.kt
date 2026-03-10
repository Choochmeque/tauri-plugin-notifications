package app.tauri.notification

import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class JSObjectUtilsTest {

    // --- putValueToJSObject primitive type tests ---

    @Test
    fun testPutString() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "key", "value")
        assertEquals("value", obj.getString("key"))
    }

    @Test
    fun testPutInt() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "n", 42)
        assertEquals(42, obj.getInt("n"))
    }

    @Test
    fun testPutLong() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "n", 9999999999L)
        assertEquals(9999999999L, obj.getLong("n"))
    }

    @Test
    fun testPutDouble() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "pi", 3.14)
        assertEquals(3.14, obj.getDouble("pi"), 0.001)
    }

    @Test
    fun testPutBoolean() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "flag", true)
        assertTrue(obj.getBoolean("flag"))
    }

    @Test
    fun testPutUnknownTypeUsesToString() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "misc", object : Any() {
            override fun toString() = "custom"
        })
        assertEquals("custom", obj.getString("misc"))
    }

    // --- putValueToJSObject nested map test ---

    @Test
    fun testPutNestedMap() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "nested", mapOf("inner" to "val", "n" to 7))

        val nested = obj.getJSObject("nested")
        assertNotNull(nested)
        assertEquals("val", nested!!.getString("inner"))
        assertEquals(7, nested.getInt("n"))
    }

    // --- putValueToJSObject list tests ---

    @Test
    fun testPutList_primitives() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "items", listOf(1, 2, 3))

        val arr = obj.getJSONArray("items")
        assertEquals(3, arr.length())
        assertEquals(1, arr.getInt(0))
        assertEquals(2, arr.getInt(1))
        assertEquals(3, arr.getInt(2))
    }

    @Test
    fun testPutList_strings() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(obj, "tags", listOf("a", "b", "c"))

        val arr = obj.getJSONArray("tags")
        assertEquals(3, arr.length())
        assertEquals("a", arr.getString(0))
    }

    @Test
    fun testPutList_withNestedMap() {
        val obj = JSObject()
        JSObjectUtils.putValueToJSObject(
            obj, "mixed",
            listOf("text", 42, mapOf("inner" to "obj"), listOf(1, 2))
        )

        val arr = obj.getJSONArray("mixed")
        assertEquals(4, arr.length())
        assertEquals("text", arr.getString(0))
        assertEquals(42, arr.getInt(1))
        assertEquals("obj", arr.getJSONObject(2).getString("inner"))
        assertEquals(2, arr.getJSONArray(3).length())
    }

    // --- convertListToJSArray tests ---

    @Test
    fun testConvertListToJSArray_booleans() {
        val arr = JSObjectUtils.convertListToJSArray(listOf(true, false, true))
        assertEquals(3, arr.length())
        assertTrue(arr.getBoolean(0))
        assertFalse(arr.getBoolean(1))
    }

    @Test
    fun testConvertListToJSArray_nullElement() {
        val list = listOf<Any?>(null, "after")
        val arr = JSObjectUtils.convertListToJSArray(list)
        assertEquals(2, arr.length())
        assertTrue(arr.isNull(0))
        assertEquals("after", arr.getString(1))
    }

    @Test
    fun testConvertListToJSArray_nestedLists() {
        val arr = JSObjectUtils.convertListToJSArray(listOf(listOf(1, 2), listOf(3, 4)))
        assertEquals(2, arr.length())
        val inner0 = arr.getJSONArray(0)
        assertEquals(1, inner0.getInt(0))
        assertEquals(2, inner0.getInt(1))
    }

    // --- jsonValueToNative tests ---

    @Test
    fun testJsonValueToNative_primitivePassThrough() {
        assertEquals("hello", JSObjectUtils.jsonValueToNative("hello"))
        assertEquals(42, JSObjectUtils.jsonValueToNative(42))
        assertEquals(3.14, JSObjectUtils.jsonValueToNative(3.14))
        assertEquals(true, JSObjectUtils.jsonValueToNative(true))
    }

    @Test
    fun testJsonValueToNative_jsonObject() {
        val json = org.json.JSONObject().apply {
            put("k", "v")
            put("n", 5)
        }
        @Suppress("UNCHECKED_CAST")
        val result = JSObjectUtils.jsonValueToNative(json) as Map<String, Any>
        assertEquals("v", result["k"])
        assertEquals(5, result["n"])
    }

    @Test
    fun testJsonValueToNative_jsonArray() {
        val json = org.json.JSONArray().apply {
            put(1)
            put("two")
        }
        @Suppress("UNCHECKED_CAST")
        val result = JSObjectUtils.jsonValueToNative(json) as List<Any>
        assertEquals(2, result.size)
        assertEquals(1, result[0])
        assertEquals("two", result[1])
    }

    @Test
    fun testJsonValueToNative_nestedObject() {
        val inner = org.json.JSONObject().apply { put("x", 99) }
        val outer = org.json.JSONObject().apply { put("inner", inner) }

        @Suppress("UNCHECKED_CAST")
        val result = JSObjectUtils.jsonValueToNative(outer) as Map<String, Any>
        @Suppress("UNCHECKED_CAST")
        val innerResult = result["inner"] as Map<String, Any>
        assertEquals(99, innerResult["x"])
    }
}

