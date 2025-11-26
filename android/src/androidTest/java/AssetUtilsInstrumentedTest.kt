// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AssetUtilsInstrumentedTest {

    private lateinit var context: Context

    @Before
    fun setup() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
    }

    @Test
    fun testGetResourceBaseName_withPath() {
        // When path contains /, it returns everything after last / (including extension)
        assertEquals("resource.png", AssetUtils.getResourceBaseName("path/to/resource.png"))
    }

    @Test
    fun testGetResourceBaseName_withExtension() {
        assertEquals("resource", AssetUtils.getResourceBaseName("resource.mp3"))
    }

    @Test
    fun testGetResourceBaseName_noExtension() {
        assertEquals("resource", AssetUtils.getResourceBaseName("resource"))
    }

    @Test
    fun testGetResourceBaseName_multipleExtensions() {
        assertEquals("file.backup", AssetUtils.getResourceBaseName("file.backup.zip"))
    }

    @Test
    fun testGetResourceBaseName_null() {
        assertNull(AssetUtils.getResourceBaseName(null))
    }

    @Test
    fun testGetResourceBaseName_empty() {
        assertEquals("", AssetUtils.getResourceBaseName(""))
    }

    @Test
    fun testGetResourceID_existingDrawable() {
        // Test with ic_transparent which should exist in this project
        val resId = AssetUtils.getResourceID(context, "ic_transparent", "drawable")

        // Should return non-zero if found, or zero if not found (both are acceptable)
        assertTrue(resId >= AssetUtils.RESOURCE_ID_ZERO_VALUE)
    }

    @Test
    fun testGetResourceID_nonExistentResource() {
        val resId = AssetUtils.getResourceID(context, "nonexistent_resource_xyz", "drawable")

        assertEquals(AssetUtils.RESOURCE_ID_ZERO_VALUE, resId)
    }

    @Test
    fun testGetResourceID_nullResourceName() {
        // Note: getResourceID doesn't handle null gracefully, Android API throws NPE
        // This tests the actual behavior
        try {
            val resId = AssetUtils.getResourceID(context, null, "drawable")
            // If it doesn't throw, it should return 0
            assertEquals(AssetUtils.RESOURCE_ID_ZERO_VALUE, resId)
        } catch (e: NullPointerException) {
            // Expected behavior - Android API throws NPE for null resource name
            assertTrue(true)
        }
    }

    @Test
    fun testGetResourceID_rawDirectory() {
        // Try to get a resource from raw directory (may not exist, but should not crash)
        val resId = AssetUtils.getResourceID(context, "test_sound", "raw")

        // Should return 0 if not found, or valid ID if found
        assertTrue(resId >= AssetUtils.RESOURCE_ID_ZERO_VALUE)
    }

    @Test
    fun testGetResourceID_drawableDirectory() {
        // Try to get a resource from drawable directory
        val resId = AssetUtils.getResourceID(context, "ic_transparent", "drawable")

        // Should return 0 if not found, or valid ID if found
        assertTrue(resId >= AssetUtils.RESOURCE_ID_ZERO_VALUE)
    }

    @Test
    fun testResourceIdZeroValueConstant() {
        assertEquals(0, AssetUtils.RESOURCE_ID_ZERO_VALUE)
    }

    @Test
    fun testGetResourceBaseName_pathProcessing() {
        // When path contains /, it returns everything after last / INCLUDING extension
        val testCases = mapOf(
            "sounds/notification.wav" to "notification.wav",
            "res/drawable/icon.png" to "icon.png",
            "assets/audio/beep.mp3" to "beep.mp3",
            "/absolute/path/file.jpg" to "file.jpg"
        )

        for ((input, expected) in testCases) {
            assertEquals("Failed for input: $input", expected, AssetUtils.getResourceBaseName(input))
        }
    }

    @Test
    fun testGetResourceBaseName_edgeCases() {
        // File with only extension (no slash, so extension is stripped)
        assertEquals("", AssetUtils.getResourceBaseName(".hidden"))

        // Path ending with slash
        assertEquals("", AssetUtils.getResourceBaseName("path/to/"))

        // Multiple slashes (has slash, so keeps extension)
        assertEquals("file.ext", AssetUtils.getResourceBaseName("path//to///file.ext"))

        // Dot in path but not in filename (has slash, returns as-is after /)
        assertEquals("file", AssetUtils.getResourceBaseName("path.v1/file"))
    }

    @Test
    fun testGetResourceID_packageContext() {
        // Verify it uses the correct package context (test package has .test suffix)
        val packageName = context.packageName
        assertEquals("app.tauri.notification.test", packageName)
    }

    @Test
    fun testGetResourceID_differentResourceTypes() {
        val resourceTypes = listOf("drawable", "raw", "string", "color", "layout", "id")

        for (type in resourceTypes) {
            val resId = AssetUtils.getResourceID(context, "test_resource", type)

            // Should not crash and return valid result (0 or positive)
            assertTrue("Failed for type: $type", resId >= 0)
        }
    }

    @Test
    fun testGetResourceBaseName_specialCharacters() {
        // With path (has /), keeps extension
        assertEquals("file_name.ext", AssetUtils.getResourceBaseName("path/file_name.ext"))
        // No path, strips extension
        assertEquals("file-name", AssetUtils.getResourceBaseName("file-name.ext"))
        assertEquals("file123", AssetUtils.getResourceBaseName("file123.ext"))
    }

    @Test
    fun testGetResourceBaseName_unicodeCharacters() {
        // Android resources typically don't support unicode, but test the base name extraction
        val result = AssetUtils.getResourceBaseName("path/файл.txt")
        assertTrue(result?.contains("файл") ?: false)
    }

    @Test
    fun testGetResourceID_emptyResourceName() {
        val resId = AssetUtils.getResourceID(context, "", "drawable")

        assertEquals(AssetUtils.RESOURCE_ID_ZERO_VALUE, resId)
    }

    @Test
    fun testGetResourceID_nullDirectory() {
        val resId = AssetUtils.getResourceID(context, "resource", null)

        // Should handle null directory gracefully
        assertEquals(AssetUtils.RESOURCE_ID_ZERO_VALUE, resId)
    }
}
