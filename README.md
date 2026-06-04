[![NPM Version](https://img.shields.io/npm/v/@choochmeque%2Ftauri-plugin-notifications-api)](https://www.npmjs.com/package/@choochmeque/tauri-plugin-notifications-api)
[![Crates.io Version](https://img.shields.io/crates/v/tauri-plugin-notifications)](https://crates.io/crates/tauri-plugin-notifications)
[![Tests](https://github.com/Choochmeque/tauri-plugin-notifications/actions/workflows/tests.yml/badge.svg)](https://github.com/Choochmeque/tauri-plugin-notifications/actions/workflows/tests.yml)
[![codecov](https://codecov.io/gh/Choochmeque/tauri-plugin-notifications/branch/main/graph/badge.svg)](https://codecov.io/gh/Choochmeque/tauri-plugin-notifications)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

# Tauri Plugin Notifications

A Tauri v2 plugin for sending notifications on desktop and mobile platforms. Send toast notifications (brief auto-expiring OS window elements) with support for rich content, scheduling, actions, channels, and push delivery via FCM, APNs, and UnifiedPush.

## Features

- Send simple and rich notifications
- Schedule notifications for specific dates or recurring intervals
- Interactive notifications with custom actions
- Notification channels (Android) for organized notifications
- Manage pending and active notifications
- Support for attachments, icons, and custom sounds
- Inbox and large text notification styles
- Group notifications with summary support
- Permission management
- Real-time notification events

## Platform Support

- **macOS**: Native notification center integration
- **Windows**: Windows notification system
- **Linux**: notify-rust with desktop notification support; push notifications via UnifiedPush
- **iOS**: User Notifications framework
- **Android**: Android notification system with channels

## Installation

Install the JavaScript package:

```bash
npm install @choochmeque/tauri-plugin-notifications-api
# or
yarn add @choochmeque/tauri-plugin-notifications-api
# or
pnpm add @choochmeque/tauri-plugin-notifications-api
```

Add the plugin to your Tauri project's `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-notifications = "0.4"
```

### Push Notifications Feature

The `push-notifications` feature is **disabled by default**. To enable push notifications support:

```toml
[dependencies]
tauri-plugin-notifications = { version = "0.4", features = ["push-notifications"] }
```

This enables:
- Firebase Cloud Messaging support on Android
- APNs (Apple Push Notification service) support on iOS and macOS
- UnifiedPush support on Linux (D-Bus distributor protocol)

**Note:** Push notifications are currently supported on iOS, Android, macOS, and Linux. Windows support is not yet available.

On Linux you also need a UnifiedPush *distributor* app installed (ntfy, NextPush, Conversations, etc. — see the [distributor list](https://unifiedpush.org/users/distributors/)). The plugin itself is stateless: any distributor selection or client token you set lives only for the current process. See [Linux UnifiedPush Setup](#linux-unifiedpush-setup) below for details.

Without this feature enabled:
- Firebase dependencies are not included in Android builds
- Push notification registration code is disabled
- The `registerForPushNotifications()` function will return an error if called

### Desktop Notification Backend (notify-rust)

The `notify-rust` feature is **enabled by default** and provides cross-platform desktop notifications using the [notify-rust](https://crates.io/crates/notify-rust) crate.

**When to use notify-rust (default):**
- Simple notifications on Linux, macOS, and Windows
- Cross-platform consistency
- Basic notification features (title, body, icon)

**When to disable notify-rust:**
- You need native Windows toast notifications with advanced features (actions, hero images, scheduling)
- You want platform-specific notification features on macOS/Windows

To disable `notify-rust` and use native platform implementations:

```toml
[dependencies]
tauri-plugin-notifications = { version = "0.4", default-features = false }
```

To disable `notify-rust` and enable push notifications:

```toml
[dependencies]
tauri-plugin-notifications = { version = "0.4", default-features = false, features = ["push-notifications"] }
```

Configure the plugin permissions in your `capabilities/default.json`:

```json
{
  "permissions": [
    "notifications:default"
  ]
}
```

Register the plugin in your Tauri app:

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notifications::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Example App

An example app is available in [`examples/notifications-demo`](examples/notifications-demo) demonstrating all plugin features:

- Permission management and push notifications (mobile)
- Basic, scheduled, and styled notifications
- Interactive notifications with action buttons
- Notification channels (Android)
- Pending and active notification management
- Event listeners with logging

**Run it:**
```bash
cd examples/notifications-demo
pnpm install
pnpm tauri dev
```

## Usage

### JavaScript/TypeScript

#### Basic Notifications

```typescript
import {
  isPermissionGranted,
  requestPermission,
  sendNotification
} from '@choochmeque/tauri-plugin-notifications-api';

// Check and request permission
let permissionGranted = await isPermissionGranted();
if (!permissionGranted) {
  const permission = await requestPermission();
  permissionGranted = permission === 'granted';
}

// Send simple notification
if (permissionGranted) {
  sendNotification('Hello from Tauri!');

  // Or with more details
  sendNotification({
    title: 'TAURI',
    body: 'Tauri is awesome!'
  });
}
```

#### Rich Notifications

```typescript
import { sendNotification } from '@choochmeque/tauri-plugin-notifications-api';

// Notification with icon and sound
await sendNotification({
  id: 1,
  title: 'New Message',
  body: 'You have a new message from John',
  icon: 'message_icon',
  sound: 'notification_sound',
  autoCancel: true
});

// Large text notification
await sendNotification({
  id: 2,
  title: 'Article',
  body: 'New article available',
  largeBody: 'This is a much longer text that will be displayed when the user expands the notification...',
  summary: 'Read more'
});

// Inbox style notification
await sendNotification({
  id: 3,
  title: 'Email',
  body: '3 new emails',
  inboxLines: [
    'Alice: Meeting at 3pm',
    'Bob: Project update',
    'Charlie: Lunch tomorrow?'
  ]
});
```

#### Scheduled Notifications

```typescript
import { sendNotification, Schedule } from '@choochmeque/tauri-plugin-notifications-api';

// Schedule notification for specific date
await sendNotification({
  title: 'Reminder',
  body: 'Time for your meeting!',
  schedule: Schedule.at(new Date(2024, 0, 15, 14, 30))
});

// Repeating notification
await sendNotification({
  title: 'Daily Reminder',
  body: 'Don\'t forget to exercise!',
  schedule: Schedule.at(new Date(2024, 0, 15, 9, 0), true)
});

// Schedule with interval
await sendNotification({
  title: 'Break Time',
  body: 'Time to take a break!',
  schedule: Schedule.interval({
    hour: 1
  })
});

// Schedule every X units
import { ScheduleEvery } from '@choochmeque/tauri-plugin-notifications-api';

await sendNotification({
  title: 'Hourly Update',
  body: 'Checking in every hour',
  schedule: Schedule.every(ScheduleEvery.Hour, 1)
});
```

#### Interactive Notifications with Actions

```typescript
import {
  sendNotification,
  registerActionTypes,
  onAction
} from '@choochmeque/tauri-plugin-notifications-api';

// Register action types
await registerActionTypes([{
  id: 'message-actions',
  actions: [
    {
      id: 'reply',
      title: 'Reply',
      input: true,
      inputPlaceholder: 'Type your reply...',
      inputButtonTitle: 'Send'
    },
    {
      id: 'mark-read',
      title: 'Mark as Read'
    },
    {
      id: 'delete',
      title: 'Delete',
      destructive: true
    }
  ]
}]);

// Send notification with actions
await sendNotification({
  title: 'New Message',
  body: 'You have a new message',
  actionTypeId: 'message-actions'
});

// Listen for action events
const unlisten = await onAction((notification) => {
  console.log('Action performed on notification:', notification);
});

// Stop listening
unlisten();
```

#### Notification Channels (Android)

```typescript
import {
  createChannel,
  channels,
  removeChannel,
  Importance,
  Visibility
} from '@choochmeque/tauri-plugin-notifications-api';

// Create a notification channel
await createChannel({
  id: 'messages',
  name: 'Messages',
  description: 'Notifications for new messages',
  importance: Importance.High,
  visibility: Visibility.Private,
  sound: 'message_sound',
  vibration: true,
  lights: true,
  lightColor: '#FF0000'
});

// Send notification to specific channel
await sendNotification({
  channelId: 'messages',
  title: 'New Message',
  body: 'You have a new message'
});

// List all channels
const channelList = await channels();

// Remove a channel
await removeChannel('messages');
```

#### Managing Notifications

```typescript
import {
  pending,
  active,
  cancel,
  cancelAll,
  removeActive,
  removeAllActive
} from '@choochmeque/tauri-plugin-notifications-api';

// Get pending notifications
const pendingNotifications = await pending();

// Cancel specific pending notifications
await cancel([1, 2, 3]);

// Cancel all pending notifications
await cancelAll();

// Get active notifications
const activeNotifications = await active();

// Remove specific active notifications
await removeActive([
  { id: 1 },
  { id: 2, tag: 'message' }
]);

// Remove all active notifications
await removeAllActive();
```

#### Notification Events

```typescript
import { onNotificationReceived } from '@choochmeque/tauri-plugin-notifications-api';

// Listen for notifications received
const unlisten = await onNotificationReceived((notification) => {
  console.log('Notification received:', notification);
});

// Stop listening
unlisten();
```

#### Push Notifications (Mobile)

```typescript
import { registerForPushNotifications } from '@choochmeque/tauri-plugin-notifications-api';

// Register for push notifications and get device token
try {
  const token = await registerForPushNotifications();
  console.log('Push token:', token);
  // Send this token to your server to send push notifications
} catch (error) {
  console.error('Failed to register for push notifications:', error);
}
```

### Rust

```rust
use tauri_plugin_notifications::{NotificationsExt, Schedule, ScheduleEvery};

// Send simple notification
app.notifications()
    .builder()
    .title("Hello")
    .body("This is a notification from Rust!")
    .show()?;

// Send rich notification
app.notifications()
    .builder()
    .id(1)
    .title("New Message")
    .body("You have a new message")
    .icon("message_icon")
    .sound("notification_sound")
    .auto_cancel()
    .show()?;

// Scheduled notification
app.notifications()
    .builder()
    .title("Reminder")
    .body("Time for your meeting!")
    .schedule(Schedule::at(date_time, false, false))
    .show()?;

// Notification with attachments
use tauri_plugin_notifications::Attachment;

app.notifications()
    .builder()
    .title("Photo Shared")
    .body("Check out this image!")
    .attachment(Attachment {
        id: "image1".to_string(),
        url: "file:///path/to/image.jpg".to_string(),
    })
    .show()?;
```

## API Reference

### `isPermissionGranted()`
Checks if the permission to send notifications is granted.

**Returns:** `Promise<boolean>`

### `requestPermission()`
Requests the permission to send notifications.

**Returns:** `Promise<'granted' | 'denied' | 'default'>`

### `registerForPushNotifications()`
Registers the app for push notifications. On Android this retrieves the FCM device token; on iOS this requests permission and registers for remote notifications; on Linux this registers with the selected UnifiedPush distributor.

**Returns:** `Promise<string>` — a platform-specific identifier:
- iOS: APNs device token
- Android: FCM device token
- Linux: UnifiedPush endpoint URL (the URL your backend POSTs payloads to)

### `listDistributors()` **(Linux / UnifiedPush only)**
Lists every running UnifiedPush distributor by its D-Bus bus name (e.g. `org.unifiedpush.Distributor.ntfy`). Returns an empty array when none is installed — that's the signal to ask the user to install one from <https://unifiedpush.org/users/distributors/>.

Throws on non-Linux platforms because the underlying Tauri command isn't registered there.

**Returns:** `Promise<string[]>`

### `setDistributor(name: string)` **(Linux / UnifiedPush only)**
Pins the distributor used on the next `registerForPushNotifications()` call. **Must be called before `registerForPushNotifications()`** — calling it after a successful register has no effect on the existing endpoint; to switch distributors, unregister and register again.

The selection is **not persisted** across launches — if the host app wants to remember the user's choice, store it and re-apply on startup. If never called, the first entry from `listDistributors()` is used.

Rejects if `name` isn't currently on the bus. Throws on non-Linux platforms.

### `setToken(token: string)` **(Linux / UnifiedPush only)**
Sets the UnifiedPush client token used on subsequent `registerForPushNotifications()` calls. **Must be called before `registerForPushNotifications()`** — calling it after a successful register has no effect on the existing endpoint.

The endpoint URL the distributor returns is derived from `(app_identifier, client_token)`, so passing the same token across launches yields the same endpoint URL. The token is **not persisted** — the host app must store it and re-apply on startup if it wants endpoint stability.

If never called, a fresh UUID is generated on each register call (FCM/APNs-style token rotation — the app pushes the new endpoint URL to its backend each time).

Rejects if `token` is empty. Throws on non-Linux platforms.

### `sendNotification(options: Options | string)`
Sends a notification to the user. Can be called with a simple string for the title or with a detailed options object.

**Parameters:**
- `options`: Notification options or title string
  - `id`: Notification identifier (32-bit integer)
  - `channelId`: Channel identifier (Android)
  - `title`: Notification title
  - `body`: Notification body
  - `schedule`: Schedule for delayed or recurring notifications
  - `largeBody`: Multiline text content
  - `summary`: Detail text for large notifications
  - `actionTypeId`: Action type identifier
  - `group`: Group identifier
  - `groupSummary`: Mark as group summary (Android)
  - `sound`: Sound resource name
  - `inboxLines`: Array of lines for inbox style (max 5)
  - `icon`: Notification icon
  - `largeIcon`: Large icon (Android)
  - `iconColor`: Icon color (Android)
  - `attachments`: Array of attachments
  - `extra`: Extra payload data
  - `ongoing`: Non-dismissible notification (Android)
  - `autoCancel`: Auto-cancel on click
  - `silent`: Silent notification (iOS)
  - `visibility`: Notification visibility
  - `number`: Number of items (Android)

### `registerActionTypes(types: ActionType[])`
Register actions that are performed when the user clicks on the notification.

**Parameters:**
- `types`: Array of action type objects with:
  - `id`: Action type identifier
  - `actions`: Array of action objects
    - `id`: Action identifier
    - `title`: Action title
    - `requiresAuthentication`: Requires device unlock
    - `foreground`: Opens app in foreground
    - `destructive`: Destructive action style
    - `input`: Enable text input
    - `inputButtonTitle`: Input button label
    - `inputPlaceholder`: Input placeholder text

### `pending()`
Retrieves the list of pending notifications.

**Returns:** `Promise<PendingNotification[]>`

### `cancel(notifications: number[])`
Cancels the pending notifications with the given list of identifiers.

### `cancelAll()`
Cancels all pending notifications.

### `active()`
Retrieves the list of active notifications.

**Returns:** `Promise<ActiveNotification[]>`

### `removeActive(notifications: Array<{ id: number; tag?: string }>)`
Removes the active notifications with the given list of identifiers.

### `removeAllActive()`
Removes all active notifications.

### `createChannel(channel: Channel)`
Creates a notification channel (Android).

**Parameters:**
- `channel`: Channel configuration
  - `id`: Channel identifier
  - `name`: Channel name
  - `description`: Channel description
  - `sound`: Sound resource name
  - `lights`: Enable notification light
  - `lightColor`: Light color
  - `vibration`: Enable vibration
  - `importance`: Importance level (None, Min, Low, Default, High)
  - `visibility`: Visibility level (Secret, Private, Public)

### `removeChannel(id: string)`
Removes the channel with the given identifier.

### `channels()`
Retrieves the list of notification channels.

**Returns:** `Promise<Channel[]>`

### `onNotificationReceived(callback: (notification: Options) => void)`
Listens for notification received events.

**Returns:** `Promise<PluginListener>` with `unlisten()` method

### `onAction(callback: (notification: Options) => void)`
Listens for notification action performed events.

**Returns:** `Promise<PluginListener>` with `unlisten()` method

## Platform Differences

### Desktop (macOS, Windows, Linux)
- Uses native notification systems
- Actions support varies by platform
- Limited scheduling capabilities on some platforms
- Channels not applicable (Android-specific)
- Linux additionally supports server-driven push via UnifiedPush (see [Linux UnifiedPush Setup](#linux-unifiedpush-setup))

### iOS
- Requires permission request
- Rich notifications with attachments
- Action support with input options
- Silent notifications available
- Group notifications (thread identifiers)

### Android
- Notification channels required for Android 8.0+
- Full scheduling support
- Rich notification styles (inbox, large text)
- Ongoing notifications for background tasks
- Detailed importance and visibility controls
- Custom sounds, vibration, and lights

## Platform Setup

### iOS Setup

1. The plugin automatically configures notification capabilities
2. Add notification sounds to your Xcode project if needed:
   - Add sound files to your iOS project
   - Place in app bundle
   - Reference by filename (without extension)

### Android Setup

1. The plugin automatically includes required permissions
2. For custom sounds:
   - Place sound files in `res/raw/` folder
   - Reference by filename (without extension)
3. For custom icons:
   - Place icons in `res/drawable/` folder
   - Reference by filename (without extension)
4. **For push notifications (FCM)** - These steps must be done in your Tauri app project:
   - Create a Firebase project at [Firebase Console](https://console.firebase.google.com/)
   - Download the `google-services.json` file from Firebase Console
   - Place `google-services.json` in your Tauri app's `gen/android/app/` directory
   - Add the Google Services classpath to your app's `gen/android/build.gradle.kts`:
     ```kotlin
     buildscript {
         repositories {
             google()
             mavenCentral()
         }
         dependencies {
             classpath("com.google.gms:google-services:4.4.2")
         }
     }
     ```
   - Apply the plugin at the bottom of `gen/android/app/build.gradle.kts`:
     ```kotlin
     apply(plugin = "com.google.gms.google-services")
     ```
   - The notification plugin already includes the Firebase Cloud Messaging dependency when the `push-notifications` feature is enabled

### Linux UnifiedPush Setup

UnifiedPush is a federated push protocol where a user-installed *distributor* app delivers messages to your app over D-Bus. The plugin implements the *connector* side and exposes the standard `registerForPushNotifications()` flow.

1. **Enable the `push-notifications` feature** in your `Cargo.toml` (this pulls in the `zbus` / `tokio` / `uuid` deps on Linux).
2. **Install a distributor**. The user picks one (or you ship one with your app):
   - [ntfy](https://ntfy.sh/) — minimal, self-hostable, has a Linux desktop client
   - [NextPush](https://nextpush.unifiedpush.org/) — backed by a Nextcloud server
   - [Conversations](https://conversations.im/) — XMPP-based
   - Full list: <https://unifiedpush.org/users/distributors/>
3. **Register from JS**:
   ```typescript
   import { registerForPushNotifications, listDistributors, setDistributor, setToken } from '@choochmeque/tauri-plugin-notifications-api';

   const distributors = await listDistributors();
   if (distributors.length === 0) {
     // prompt the user to install a distributor
     return;
   }
   await setDistributor(distributors[0]);
   const storedToken = localStorage.getItem('up-client-token') ?? crypto.randomUUID();
   localStorage.setItem('up-client-token', storedToken);
   await setToken(storedToken);
   const endpoint = await registerForPushNotifications();
   // POST `endpoint` to your backend so it can deliver pushes
   ```

#### Receiving pushes while the app is running

Incoming UnifiedPush messages are handled automatically:

- A system toast is shown via `notify-rust` (if that feature is enabled — it is by default).
- The same `onNotificationReceived` listener that handles local notifications fires with `source: "push"`:

  ```typescript
  import { onNotificationReceived } from '@choochmeque/tauri-plugin-notifications-api';

  const unlisten = await onNotificationReceived((n) => {
    if (n.source === 'push') {
      console.log('UnifiedPush message:', n.title, n.body, n.extra);
    }
  });
  ```

The plugin best-effort-parses the message bytes as JSON and extracts `title`, `body` (or `message`), and `data` (or `extra`). Non-JSON payloads land in `body` as a plain string. Binary payloads end up in `extra` as a `<binary N bytes>` marker.

#### Receiving pushes when the app is closed (optional)

UnifiedPush delivers messages by D-Bus method call on the app's bus name. If the app isn't running when a push arrives, the distributor relies on D-Bus session activation to launch it.

To enable that, ship a `.service` activation file at `~/.local/share/dbus-1/services/<app-identifier>.service`:

```ini
[D-BUS Service]
Name=com.example.MyApp
Exec=/usr/bin/my-app
```

Replace `Name` with your Tauri app's identifier (the `identifier` field from `tauri.conf.json`) and `Exec` with the absolute path to the installed binary. Without this file, pushes are only delivered while the app is already running.

The plugin does **not** install this file for you — it's a packaging/deployment decision. Distros and packagers typically install it from the `.deb` / `.rpm` / Flatpak manifest.

**Important caveat about cold-start delivery:** the plugin's UnifiedPush connector (the D-Bus service that owns your app identifier) is **lazily initialized** on the first call to `listDistributors()`, `setDistributor()`, `setToken()`, or `registerForPushNotifications()` from the JS layer. If your app is launched by D-Bus activation purely to handle a push, the distributor's method call may race the lazy init and arrive before the connector is registered — the call then fails silently. To make activation-driven delivery reliable, trigger the init eagerly in your Rust `setup()` (e.g. call `notifications.list_distributors()` once before returning), or call `listDistributors()` from JS as early as your app starts. Until you do, treat the `.service` activation as best-effort.

## Testing

### Desktop
- Notifications appear in the system notification center
- Test different notification types and interactions
- Verify notification persistence and dismissal

### iOS
- Test on physical devices (simulator support is limited)
- Request permissions before sending notifications
- Test scheduled notifications with different intervals
- Verify action handling and notification grouping

### Android
- Create and test notification channels
- Test different importance levels and visibility settings
- Verify scheduled notifications work with device sleep
- Test ongoing notifications for background tasks
- Verify notification styles (inbox, large text, etc.)

## Troubleshooting

### Notifications not appearing
- Verify permissions are granted
- On Android, ensure notification channel exists
- Check system notification settings
- Verify notification ID is unique

### Scheduled notifications not firing
- Check device power settings (battery optimization)
- On Android, use `allowWhileIdle` for critical notifications
- Verify schedule time is in the future

### Actions not working
- Ensure action types are registered before sending notification
- Verify action IDs match between registration and handling
- Check platform-specific action support

## License

[MIT](LICENSE)
