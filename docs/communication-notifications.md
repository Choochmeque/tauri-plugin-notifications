# Communication Notifications (iOS)

This guide explains how to enable Communication Notifications so your push notifications can show a sender avatar in the banner on iOS 15+.

These steps are required by Apple for messaging/VoIP apps. The plugin cannot add these capabilities automatically; each app must opt in and add a Notification Service Extension.

## What You Get
- Sender avatar in the notification banner (when iOS accepts it as a communication notification)
- System recognizes the notification as a real message

## Requirements
- iOS 15+
- Push notifications (APNs)
- Notification Service Extension
- "Communication Notifications" capability enabled in Xcode

## 1. Add the Extension Template

Copy the extension template into your iOS project:

- `ios/Extensions/CommunicationNotifications/NotificationService.swift`
- `ios/Extensions/CommunicationNotifications/Info.plist`

If you use XcodeGen or a custom iOS project layout, place these files under a new extension target named something like `YourApp_NotificationService`.

## 2. Enable Capabilities

In Xcode:
1. Select your app target
2. Go to "Signing & Capabilities"
3. Add "Communication Notifications"
4. Ensure "Push Notifications" is enabled

## 3. Add NSUserActivityTypes

Add `NSUserActivityTypes` to your app Info.plist:

```xml
<key>NSUserActivityTypes</key>
<array>
  <string>INSendMessageIntent</string>
</array>
```

## 4. Update Bundle IDs

Your extension must have its own bundle ID, usually:

```
com.your.bundle.id.notificationservice
```

It must be under the same Team ID and provisioning profile as your app.

## 5. APNs Payload

Your push payload must include `mutable-content: 1` so the extension can run:

```json
{
  "aps": {
    "alert": {
      "title": "Alice",
      "body": "Hey, are you coming?"
    },
    "mutable-content": 1
  },
  "sender_name": "Alice",
  "sender_id": "alice-123",
  "sender_avatar_url": "https://github.com/benjamincanac.png",
  "conversation_id": "chat-42"
}
```

## 6. Notes
- iOS only shows the sender avatar if the notification is recognized as a real message.
- The service extension has only a few seconds to download the avatar and attach the intent.
- If avatar download fails, iOS falls back to the app icon.

