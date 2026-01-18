package app.tauri.notification

import app.tauri.Logger
import app.tauri.plugin.JSObject
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

private const val ATTACHMENT_TAG = "NotificationAttachment"

class NotificationAttachment {
  var id: String? = null
  var url: String? = null
  var options: JSONObject? = null

  companion object {
    fun getAttachments(notification: JSObject): List<NotificationAttachment> {
      val attachmentsList: MutableList<NotificationAttachment> = ArrayList()
      var attachments: JSONArray? = null
      try {
        attachments = notification.getJSONArray("attachments")
      } catch (e: Exception) {
        Logger.debug(Logger.tags(ATTACHMENT_TAG), "No attachments found in notification: ${e.message}")
      }
      if (attachments != null) {
        for (i in 0 until attachments.length()) {
          val newAttachment = NotificationAttachment()
          var jsonObject: JSONObject? = null
          try {
            jsonObject = attachments.getJSONObject(i)
          } catch (e: JSONException) {
            Logger.error(Logger.tags(ATTACHMENT_TAG), "Failed to get attachment object at index $i: ${e.message}", e)
          }
          if (jsonObject != null) {
            var jsObject: JSObject? = null
            try {
              jsObject = JSObject.fromJSONObject(jsonObject)
            } catch (e: JSONException) {
              Logger.error(Logger.tags(ATTACHMENT_TAG), "Failed to convert attachment JSON object: ${e.message}", e)
            }
            newAttachment.id = jsObject!!.getString("id")
            newAttachment.url = jsObject.getString("url")
            try {
              newAttachment.options = jsObject.getJSONObject("options")
            } catch (e: JSONException) {
              Logger.debug(Logger.tags(ATTACHMENT_TAG), "No options found for attachment: ${e.message}")
            }
            attachmentsList.add(newAttachment)
          }
        }
      }
      return attachmentsList
    }
  }
}