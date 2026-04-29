package app.tauri.notification

import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import org.json.JSONArray
import org.json.JSONObject

/**
 * Shared helpers for recursively converting native Kotlin values (Maps, Lists, primitives)
 * into [JSObject] / [JSArray] structures understood by the Tauri plugin bridge.
 *
 * Both [NotificationPlugin] and [TauriUnifiedPushMessagingService] need to convert
 * the push-data maps they receive into JS-bridge types; keeping the logic in one
 * place prevents the two copies from drifting apart.
 */
internal object JSObjectUtils {

    /**
     * Recursively put [value] into [target] under [key].
     * Handles String, Int, Long, Double, Boolean, Map, and List types.
     */
    fun putValueToJSObject(target: JSObject, key: String, value: Any) {
        when (value) {
            is String -> target.put(key, value)
            is Int -> target.put(key, value)
            is Long -> target.put(key, value)
            is Double -> target.put(key, value)
            is Boolean -> target.put(key, value)
            is Map<*, *> -> {
                val nestedObj = JSObject()
                @Suppress("UNCHECKED_CAST")
                val map = value as Map<String, Any>
                for ((k, v) in map) {
                    putValueToJSObject(nestedObj, k, v)
                }
                target.put(key, nestedObj)
            }
            is List<*> -> target.put(key, convertListToJSArray(value))
            else -> target.put(key, value.toString())
        }
    }

    /**
     * Recursively convert a [List] into a [JSArray], handling nested maps, lists, and primitives.
     */
    fun convertListToJSArray(list: List<*>): JSArray {
        val arr = JSArray()
        for (item in list) {
            when (item) {
                is String -> arr.put(item)
                is Int -> arr.put(item)
                is Long -> arr.put(item)
                is Double -> arr.put(item)
                is Boolean -> arr.put(item)
                is Map<*, *> -> {
                    val nestedObj = JSObject()
                    @Suppress("UNCHECKED_CAST")
                    val map = item as Map<String, Any>
                    for ((k, v) in map) {
                        putValueToJSObject(nestedObj, k, v)
                    }
                    arr.put(nestedObj)
                }
                is List<*> -> arr.put(convertListToJSArray(item))
                null -> arr.put(JSONObject.NULL)
                else -> arr.put(item.toString())
            }
        }
        return arr
    }

    /**
     * Recursively convert a raw [JSONObject] value into a native Kotlin type
     * (Map for objects, List for arrays, or the primitive itself).
     */
    fun jsonValueToNative(value: Any): Any {
        return when (value) {
            is JSONObject -> {
                val map = mutableMapOf<String, Any>()
                for (key in value.keys()) {
                    map[key] = jsonValueToNative(value.get(key))
                }
                map
            }
            is JSONArray -> {
                val list = mutableListOf<Any>()
                for (i in 0 until value.length()) {
                    list.add(jsonValueToNative(value.get(i)))
                }
                list
            }
            else -> value
        }
    }
}

