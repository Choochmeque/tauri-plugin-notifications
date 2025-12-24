// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import app.tauri.plugin.JSObject
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class NotificationAttachmentTest {

    @Test
    fun testGetAttachments_singleAttachment() {
        val attachmentJson = JSONObject()
        attachmentJson.put("id", "attachment1")
        attachmentJson.put("url", "file:///path/to/image.jpg")

        val attachmentsArray = JSONArray()
        attachmentsArray.put(attachmentJson)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(1, result.size)
        assertEquals("attachment1", result[0].id)
        assertEquals("file:///path/to/image.jpg", result[0].url)
        assertNull(result[0].options)
    }

    @Test
    fun testGetAttachments_multipleAttachments() {
        val attachment1 = JSONObject()
        attachment1.put("id", "attachment1")
        attachment1.put("url", "file:///path/to/image1.jpg")

        val attachment2 = JSONObject()
        attachment2.put("id", "attachment2")
        attachment2.put("url", "file:///path/to/image2.png")

        val attachment3 = JSONObject()
        attachment3.put("id", "attachment3")
        attachment3.put("url", "file:///path/to/video.mp4")

        val attachmentsArray = JSONArray()
        attachmentsArray.put(attachment1)
        attachmentsArray.put(attachment2)
        attachmentsArray.put(attachment3)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(3, result.size)
        assertEquals("attachment1", result[0].id)
        assertEquals("attachment2", result[1].id)
        assertEquals("attachment3", result[2].id)
    }

    @Test
    fun testGetAttachments_withOptions() {
        val options = JSONObject()
        options.put("thumbnail", true)
        options.put("quality", 0.8)

        val attachmentJson = JSONObject()
        attachmentJson.put("id", "attachment1")
        attachmentJson.put("url", "file:///path/to/image.jpg")
        attachmentJson.put("options", options)

        val attachmentsArray = JSONArray()
        attachmentsArray.put(attachmentJson)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(1, result.size)
        assertEquals("attachment1", result[0].id)
        assertNotNull(result[0].options)
        assertEquals(true, result[0].options?.getBoolean("thumbnail"))
        assertEquals(0.8, result[0].options?.getDouble("quality") ?: 0.0, 0.001)
    }

    @Test
    fun testGetAttachments_emptyArray() {
        val attachmentsArray = JSONArray()

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(0, result.size)
    }

    @Test
    fun testGetAttachments_noAttachmentsField() {
        val notification = JSObject()

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(0, result.size)
    }

    @Test
    fun testGetAttachments_invalidArrayElement() {
        val attachmentsArray = JSONArray()
        attachmentsArray.put("invalid string")
        attachmentsArray.put(123)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(0, result.size)
    }

    @Test
    fun testGetAttachments_mixedValidInvalid() {
        val validAttachment = JSONObject()
        validAttachment.put("id", "valid")
        validAttachment.put("url", "file:///valid.jpg")

        val attachmentsArray = JSONArray()
        attachmentsArray.put(validAttachment)
        attachmentsArray.put("invalid")
        attachmentsArray.put(validAttachment)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(2, result.size)
        assertEquals("valid", result[0].id)
        assertEquals("valid", result[1].id)
    }

    @Test
    fun testGetAttachments_missingIdField() {
        val attachmentJson = JSONObject()
        attachmentJson.put("url", "file:///path/to/image.jpg")

        val attachmentsArray = JSONArray()
        attachmentsArray.put(attachmentJson)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(1, result.size)
        assertTrue(result[0].id.isNullOrEmpty())
        assertEquals("file:///path/to/image.jpg", result[0].url)
    }

    @Test
    fun testGetAttachments_missingUrlField() {
        val attachmentJson = JSONObject()
        attachmentJson.put("id", "attachment1")

        val attachmentsArray = JSONArray()
        attachmentsArray.put(attachmentJson)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(1, result.size)
        assertEquals("attachment1", result[0].id)
        assertTrue(result[0].url.isNullOrEmpty())
    }

    @Test
    fun testGetAttachments_emptyStrings() {
        val attachmentJson = JSONObject()
        attachmentJson.put("id", "")
        attachmentJson.put("url", "")

        val attachmentsArray = JSONArray()
        attachmentsArray.put(attachmentJson)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(1, result.size)
        assertEquals("", result[0].id)
        assertEquals("", result[0].url)
    }

    @Test
    fun testGetAttachments_differentUrlFormats() {
        val attachment1 = JSONObject()
        attachment1.put("id", "http")
        attachment1.put("url", "http://example.com/image.jpg")

        val attachment2 = JSONObject()
        attachment2.put("id", "https")
        attachment2.put("url", "https://example.com/image.jpg")

        val attachment3 = JSONObject()
        attachment3.put("id", "file")
        attachment3.put("url", "file:///local/path/image.jpg")

        val attachment4 = JSONObject()
        attachment4.put("id", "content")
        attachment4.put("url", "content://media/external/images/1")

        val attachmentsArray = JSONArray()
        attachmentsArray.put(attachment1)
        attachmentsArray.put(attachment2)
        attachmentsArray.put(attachment3)
        attachmentsArray.put(attachment4)

        val notification = JSObject()
        notification.put("attachments", attachmentsArray)

        val result = NotificationAttachment.getAttachments(notification)

        assertEquals(4, result.size)
        assertEquals("http://example.com/image.jpg", result[0].url)
        assertEquals("https://example.com/image.jpg", result[1].url)
        assertEquals("file:///local/path/image.jpg", result[2].url)
        assertEquals("content://media/external/images/1", result[3].url)
    }

    @Test
    fun testNotificationAttachment_properties() {
        val attachment = NotificationAttachment()
        attachment.id = "test_id"
        attachment.url = "test_url"

        val options = JSONObject()
        options.put("key", "value")
        attachment.options = options

        assertEquals("test_id", attachment.id)
        assertEquals("test_url", attachment.url)
        assertEquals("value", attachment.options?.getString("key"))
    }

    @Test
    fun testNotificationAttachment_nullProperties() {
        val attachment = NotificationAttachment()

        assertNull(attachment.id)
        assertNull(attachment.url)
        assertNull(attachment.options)
    }
}
