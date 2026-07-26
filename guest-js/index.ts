/**
 * Send toast notifications (brief auto-expiring OS window element) to your user.
 * Can also be used with the Notification Web API.
 *
 * @module
 */

import {
  invoke,
  type PluginListener,
  addPluginListener,
} from "@tauri-apps/api/core";

export type { PermissionState } from "@tauri-apps/api/core";

/**
 * Options to send a notification.
 */
interface Options {
  /**
   * The notification identifier to reference this object later. Must be a 32-bit integer.
   */
  id?: number;
  /**
   * Identifier of the {@link Channel} that delivers this notification.
   *
   * If the channel does not exist, the notification won't fire.
   * Make sure the channel exists with {@link channels} and {@link createChannel}.
   */
  channelId?: string;
  /**
   * Notification title.
   */
  title: string;
  /**
   * Optional notification body.
   * */
  body?: string;
  /**
   * Schedule this notification to fire on a later time or a fixed interval.
   */
  schedule?: Schedule;
  /**
   * Multiline text.
   * Changes the notification style to big text.
   * Cannot be used with `inboxLines`.
   */
  largeBody?: string;
  /**
   * Detail text for the notification with `largeBody`, `inboxLines` or `groupSummary`.
   */
  summary?: string;
  /**
   * Defines an action type for this notification.
   */
  actionTypeId?: string;
  /**
   * Identifier used to group multiple notifications.
   *
   * https://developer.apple.com/documentation/usernotifications/unmutablenotificationcontent/1649872-threadidentifier
   */
  group?: string;
  /**
   * Instructs the system that this notification is the summary of a group on Android.
   */
  groupSummary?: boolean;
  /**
   * The sound resource name. Only available on mobile.
   */
  sound?: string;
  /**
   * List of lines to add to the notification.
   * Changes the notification style to inbox.
   * Cannot be used with `largeBody`.
   *
   * Only supports up to 5 lines.
   */
  inboxLines?: string[];
  /**
   * Notification icon.
   *
   * On Android the icon must be placed in the app's `res/drawable` folder.
   */
  icon?: string;
  /**
   * Notification large icon (Android).
   *
   * The icon must be placed in the app's `res/drawable` folder.
   */
  largeIcon?: string;
  /**
   * Icon color on Android.
   */
  iconColor?: string;
  /**
   * Notification attachments.
   */
  attachments?: Attachment[];
  /**
   * Extra payload to store in the notification.
   */
  extra?: Record<string, unknown>;
  /**
   * If true, the notification cannot be dismissed by the user on Android.
   *
   * An application service must manage the dismissal of the notification.
   * It is typically used to indicate a background task that is pending (e.g. a file download)
   * or the user is engaged with (e.g. playing music).
   */
  ongoing?: boolean;
  /**
   * Automatically cancel the notification when the user clicks on it.
   */
  autoCancel?: boolean;
  /**
   * Changes the notification presentation to be silent on iOS (no badge, no sound, not listed).
   */
  silent?: boolean;
  /**
   * The source of the notification. Only present in `onNotificationReceived` callbacks.
   * - `"push"` — notification received from a remote push (FCM/APNs).
   * - `"local"` — notification created locally (immediate or scheduled).
   */
  source?: "push" | "local";
  /**
   * Notification visibility.
   */
  visibility?: Visibility;
  /**
   * Sets the number of items this notification represents on Android.
   */
  number?: number;
}

/**
 * Interval configuration for scheduling notifications.
 */
interface ScheduleInterval {
  /** Year component of the schedule interval. */
  year?: number;
  /** Month component of the schedule interval. */
  month?: number;
  /** Day component of the schedule interval. */
  day?: number;
  /**
   * Weekday component of the schedule interval.
   * - 1 - Sunday
   * - 2 - Monday
   * - 3 - Tuesday
   * - 4 - Wednesday
   * - 5 - Thursday
   * - 6 - Friday
   * - 7 - Saturday
   */
  weekday?: number;
  /** Hour component of the schedule interval. */
  hour?: number;
  /** Minute component of the schedule interval. */
  minute?: number;
  /** Second component of the schedule interval. */
  second?: number;
}

/**
 * Predefined intervals for repeating notifications.
 */
enum ScheduleEvery {
  Year = "year",
  Month = "month",
  TwoWeeks = "twoWeeks",
  Week = "week",
  Day = "day",
  Hour = "hour",
  Minute = "minute",
  /**
   * Not supported on iOS.
   */
  Second = "second",
}

/**
 * Schedule configuration for notifications.
 */
class Schedule {
  /** Schedule a notification at a specific date and time. */
  at:
    | {
        date: Date;
        repeating: boolean;
        allowWhileIdle: boolean;
      }
    | undefined;

  /** Schedule a notification using an interval configuration. */
  interval:
    | {
        interval: ScheduleInterval;
        allowWhileIdle: boolean;
      }
    | undefined;

  /** Schedule a notification to repeat at regular intervals. */
  every:
    | {
        interval: ScheduleEvery;
        count: number;
        allowWhileIdle: boolean;
      }
    | undefined;

  /**
   * Creates a schedule to fire at a specific date and time.
   *
   * @param date - The date and time to fire the notification.
   * @param repeating - Whether to repeat the notification at the same time daily.
   * @param allowWhileIdle - On Android, allows notification to fire even when the device is in idle mode.
   * @returns A new Schedule instance.
   */
  static at(date: Date, repeating = false, allowWhileIdle = false): Schedule {
    return {
      at: { date, repeating, allowWhileIdle },
      interval: undefined,
      every: undefined,
    };
  }

  /**
   * Creates a schedule using an interval configuration.
   *
   * @param interval - The interval configuration specifying when to fire.
   * @param allowWhileIdle - On Android, allows notification to fire even when the device is in idle mode.
   * @returns A new Schedule instance.
   */
  static interval(
    interval: ScheduleInterval,
    allowWhileIdle = false,
  ): Schedule {
    return {
      at: undefined,
      interval: { interval, allowWhileIdle },
      every: undefined,
    };
  }

  /**
   * Creates a schedule to repeat at regular intervals.
   *
   * @param kind - The type of interval (year, month, week, day, hour, minute, second).
   * @param count - The number of intervals between notifications.
   * @param allowWhileIdle - On Android, allows notification to fire even when the device is in idle mode.
   * @returns A new Schedule instance.
   */
  static every(
    kind: ScheduleEvery,
    count: number,
    allowWhileIdle = false,
  ): Schedule {
    return {
      at: undefined,
      interval: undefined,
      every: { interval: kind, count, allowWhileIdle },
    };
  }
}

/**
 * Attachment of a notification.
 */
interface Attachment {
  /** Attachment identifier. */
  id: string;
  /** Attachment URL. Accepts the `asset` and `file` protocols. */
  url: string;
}

/**
 * An action that can be performed from a notification.
 */
interface Action {
  /** Unique identifier for the action. */
  id: string;
  /** The title text displayed for the action. */
  title: string;
  /** Whether the action requires device authentication (iOS). */
  requiresAuthentication?: boolean;
  /** Whether the action should launch the app in the foreground. */
  foreground?: boolean;
  /** Whether the action is destructive (displayed in red on iOS). */
  destructive?: boolean;
  /** Whether the action allows text input. */
  input?: boolean;
  /** The title for the input button when `input` is true. */
  inputButtonTitle?: string;
  /** Placeholder text for the input field when `input` is true. */
  inputPlaceholder?: string;
}

/**
 * A group of related actions that can be performed from a notification.
 */
interface ActionType {
  /** The identifier of this action type. */
  id: string;
  /** The list of associated actions. */
  actions: Action[];
  /** Placeholder text shown in place of the notification body when previews are hidden (iOS). */
  hiddenPreviewsBodyPlaceholder?: string;
  /** Whether to include a custom dismiss action (iOS). */
  customDismissAction?: boolean;
  /** Whether the notification can be displayed in CarPlay (iOS). */
  allowInCarPlay?: boolean;
  /** Whether to show the title when previews are hidden (iOS). */
  hiddenPreviewsShowTitle?: boolean;
  /** Whether to show the subtitle when previews are hidden (iOS). */
  hiddenPreviewsShowSubtitle?: boolean;
}

/**
 * Represents a scheduled notification that has not yet been delivered.
 */
interface PendingNotification {
  /** Notification identifier. */
  id: number;
  /** Notification title. */
  title?: string;
  /** Notification body. */
  body?: string;
  /** The schedule configuration for this notification. */
  schedule: Schedule;
}

/**
 * Represents a notification that is currently displayed.
 */
interface ActiveNotification {
  /** Notification identifier. */
  id: number;
  /** Optional tag for the notification. */
  tag?: string;
  /** Notification title. */
  title?: string;
  /** Notification body. */
  body?: string;
  /** Group identifier for this notification. */
  group?: string;
  /** Whether this notification is a group summary. */
  groupSummary: boolean;
  /** Additional string data attached to the notification. */
  data: Record<string, string>;
  /** Extra payload stored in the notification. */
  extra: Record<string, unknown>;
  /** List of attachments for this notification. */
  attachments: Attachment[];
  /** The action type identifier for this notification. */
  actionTypeId?: string;
  /** The schedule configuration if this was a scheduled notification. */
  schedule?: Schedule;
  /** The sound resource name. */
  sound?: string;
}

/** Data received when an action is performed on a notification. */
interface ActionPerformedData {
  /** Identifier of the selected action, or `tap`/`dismiss` for system actions. */
  actionId: string;
  /** Text entered for an action with `input: true`. */
  inputValue?: string;
  /** The notification, including its action type and extra metadata. */
  notification: ActionNotification;
}

/** Notification data included with an action result. Platform-specific fields are optional. */
interface ActionNotification {
  /** Numeric local notification ID, or -1 when a provider notification has no numeric ID. */
  id: number;
  title?: string;
  body?: string;
  actionTypeId?: string;
  extra?: Record<string, unknown>;
  source?: "local" | "push";
}

/**
 * The importance level of a notification channel (Android).
 */
enum Importance {
  /** Does not show notifications. */
  None = 0,
  /** Shows notifications only in the notification shade, no sound, no visual interruption. */
  Min,
  /** Shows notifications everywhere, but is not intrusive. */
  Low,
  /** Shows notifications everywhere with sound. */
  Default,
  /** Shows notifications everywhere with sound and heads-up display. */
  High,
}

/**
 * The visibility of a notification on the lock screen (Android).
 */
enum Visibility {
  /** Do not show any part of this notification on the lock screen. */
  Secret = -1,
  /** Show the notification, but hide sensitive content on the lock screen. */
  Private,
  /** Show the entire notification on the lock screen. */
  Public,
}

/**
 * A notification channel (Android).
 */
interface Channel {
  /** Channel identifier. */
  id: string;
  /** Channel name shown to the user. */
  name: string;
  /** Channel description shown to the user. */
  description?: string;
  /** Sound resource name for notifications in this channel. */
  sound?: string;
  /** Whether to show LED lights for notifications in this channel. */
  lights?: boolean;
  /** The LED light color in hex format (e.g., "#FF0000"). */
  lightColor?: string;
  /** Whether to vibrate for notifications in this channel. */
  vibration?: boolean;
  /** The importance level for notifications in this channel. */
  importance?: Importance;
  /** The visibility level on the lock screen for notifications in this channel. */
  visibility?: Visibility;
}

/**
 * Checks if the permission to send notifications is granted.
 * @example
 * ```typescript
 * import { isPermissionGranted } from '@choochmeque/tauri-plugin-notifications-api';
 * const permissionGranted = await isPermissionGranted();
 * ```
 */
async function isPermissionGranted(): Promise<boolean> {
  return await invoke("plugin:notifications|is_permission_granted");
}

/**
 * Requests the permission to send notifications.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission } from '@choochmeque/tauri-plugin-notifications-api';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * ```
 *
 * @returns A promise resolving to whether the user granted the permission or not.
 */
async function requestPermission(): Promise<NotificationPermission> {
  return await invoke("plugin:notifications|request_permission");
}

interface PushRegistration {
  deviceToken: string;
  p256dh?: string;
  auth?: string;
  instance?: string;
}

type PushProvider = "auto" | "fcm" | "unifiedpush";

/**
 * Registers the app for push notifications.
 *
 * `deviceToken` is the APNs token on iOS and the UnifiedPush endpoint URL on
 * Android/Linux. Pass a base64url VAPID public key to register against a Web
 * Push distributor; `p256dh` and `auth` are then set on the result.
 *
 * @returns A promise resolving to the {@link PushRegistration}.
 */
async function registerForPushNotifications(
  vapid?: string,
  provider: PushProvider = "auto",
): Promise<PushRegistration> {
  return await invoke("plugin:notifications|register_for_push_notifications", {
    vapid,
    provider,
  });
}

/**
 * Unregisters the app from push notifications.
 *
 * This removes the device's push notification registration and stops
 * receiving remote pushes. On Linux it calls `Unregister` on the active
 * UnifiedPush distributor.
 *
 * @example
 * ```typescript
 * import { unregisterForPushNotifications } from '@choochmeque/tauri-plugin-notifications-api';
 * await unregisterForPushNotifications();
 * console.log('Unregistered from push notifications');
 * ```
 *
 * @returns A promise resolving when unregistration is complete.
 */
async function unregisterForPushNotifications(): Promise<void> {
  await invoke("plugin:notifications|unregister_for_push_notifications");
}

/**
 * Lists currently running UnifiedPush distributors by D-Bus bus name
 * (e.g. `org.unifiedpush.Distributor.ntfy`).
 *
 * **Linux only, and only when the plugin was built with the
 * `push-notifications` Rust feature** (the Tauri commands are gated behind
 * `#[cfg(all(target_os = "linux", feature = "push-notifications"))]`).
 * Throws elsewhere because the command isn't registered.
 *
 * Returns an empty array when no UnifiedPush distributor is installed —
 * this is the signal to prompt the user to install one
 * (https://unifiedpush.org/users/distributors/).
 *
 * @example
 * ```typescript
 * import { listDistributors } from '@choochmeque/tauri-plugin-notifications-api';
 * const distributors = await listDistributors();
 * if (distributors.length === 0) {
 *   alert('Please install a UnifiedPush distributor');
 * }
 * ```
 *
 * @returns A promise resolving to the list of distributor bus names.
 * @platform linux
 */
async function listDistributors(): Promise<string[]> {
  return await invoke("plugin:notifications|list_distributors");
}

/**
 * Pins the UnifiedPush distributor used for the next
 * {@link registerForPushNotifications} call.
 *
 * **Linux only, and only when the plugin was built with the
 * `push-notifications` Rust feature.** Throws elsewhere because the
 * underlying Tauri command isn't registered.
 *
 * **Must be called before {@link registerForPushNotifications}.** Calling
 * this after a successful register has no effect on the existing endpoint
 * — to switch distributors, unregister and register again.
 *
 * The selection lives only for the current process — it is **not persisted**
 * across launches. If the host app wants to remember the user's choice, it
 * must store the name itself and re-apply it via this function on startup.
 * If never called, the first distributor returned by {@link listDistributors}
 * is used.
 *
 * @example
 * ```typescript
 * import { setDistributor } from '@choochmeque/tauri-plugin-notifications-api';
 * await setDistributor('org.unifiedpush.Distributor.ntfy');
 * ```
 *
 * @param name The distributor bus name. Must be one of the values returned
 *             by {@link listDistributors}; otherwise rejects.
 * @platform linux
 */
async function setDistributor(name: string): Promise<void> {
  await invoke("plugin:notifications|set_distributor", { name });
}

/**
 * Sets the UnifiedPush client token used on subsequent
 * {@link registerForPushNotifications} calls.
 *
 * **Linux only, and only when the plugin was built with the
 * `push-notifications` Rust feature.** Throws elsewhere because the
 * underlying Tauri command isn't registered.
 *
 * **Must be called before {@link registerForPushNotifications}.** Calling
 * this after a successful register has no effect on the existing endpoint
 * — to get a different endpoint URL, unregister and register again.
 *
 * UnifiedPush distributors derive the endpoint URL from
 * `(connector_bus_name, client_token)`, so apps that want endpoint
 * stability across launches should persist a token (any non-empty string,
 * usually a UUID) and call this on startup before registering. If never
 * called, a fresh UUID is generated on each register call and the endpoint
 * URL will be unique per launch — the same model as FCM/APNs token
 * rotation, where the app pushes the new token to its backend each time.
 *
 * The token lives only for the current process — it is **not persisted**
 * across launches.
 *
 * @example
 * ```typescript
 * import { setToken, registerForPushNotifications } from '@choochmeque/tauri-plugin-notifications-api';
 * const storedToken = localStorage.getItem('up-client-token') ?? crypto.randomUUID();
 * localStorage.setItem('up-client-token', storedToken);
 * await setToken(storedToken);
 * const endpoint = await registerForPushNotifications();
 * ```
 *
 * @param token The client token. Must be non-empty.
 * @platform linux
 */
async function setToken(token: string): Promise<void> {
  await invoke("plugin:notifications|set_token", { token });
}

/**
 * Sends a notification to the user.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission, sendNotification } from '@choochmeque/tauri-plugin-notifications-api';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * if (permissionGranted) {
 *   sendNotification('Tauri is awesome!');
 *   sendNotification({ title: 'TAURI', body: 'Tauri is awesome!' });
 * }
 * ```
 */
async function sendNotification(options: Options | string): Promise<void> {
  await invoke("plugin:notifications|notify", {
    options:
      typeof options === "string"
        ? {
            title: options,
          }
        : options,
  });
}

/**
 * Register actions that are performed when the user clicks on the notification.
 *
 * @example
 * ```typescript
 * import { registerActionTypes } from '@choochmeque/tauri-plugin-notifications-api';
 * await registerActionTypes([{
 *   id: 'tauri',
 *   actions: [{
 *     id: 'my-action',
 *     title: 'Settings'
 *   }]
 * }])
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 */
async function registerActionTypes(types: ActionType[]): Promise<void> {
  await invoke("plugin:notifications|register_action_types", { types });
}

/**
 * Retrieves the list of pending notifications.
 *
 * @example
 * ```typescript
 * import { pending } from '@choochmeque/tauri-plugin-notifications-api';
 * const pendingNotifications = await pending();
 * ```
 *
 * @returns A promise resolving to the list of pending notifications.
 */
async function pending(): Promise<PendingNotification[]> {
  return await invoke("plugin:notifications|get_pending");
}

/**
 * Cancels the pending notifications with the given list of identifiers.
 *
 * @example
 * ```typescript
 * import { cancel } from '@choochmeque/tauri-plugin-notifications-api';
 * await cancel([-34234, 23432, 4311]);
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 */
async function cancel(notifications: number[]): Promise<void> {
  await invoke("plugin:notifications|cancel", { notifications });
}

/**
 * Cancels all pending notifications.
 *
 * @example
 * ```typescript
 * import { cancelAll } from '@choochmeque/tauri-plugin-notifications-api';
 * await cancelAll();
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 */
async function cancelAll(): Promise<void> {
  await invoke("plugin:notifications|cancel_all");
}

/**
 * Retrieves the list of active notifications.
 *
 * @example
 * ```typescript
 * import { active } from '@choochmeque/tauri-plugin-notifications-api';
 * const activeNotifications = await active();
 * ```
 *
 * @returns A promise resolving to the list of active notifications.
 */
async function active(): Promise<ActiveNotification[]> {
  return await invoke("plugin:notifications|get_active");
}

/**
 * Removes the active notifications with the given list of identifiers.
 *
 * @example
 * ```typescript
 * import { removeActive } from '@choochmeque/tauri-plugin-notifications-api';
 * await removeActive([{ id: 1 }, { id: 2, tag: 'news' }]);
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 */
async function removeActive(
  notifications: Array<{ id: number; tag?: string }>,
): Promise<void> {
  await invoke("plugin:notifications|remove_active", { notifications });
}

/**
 * Removes all active notifications.
 *
 * @example
 * ```typescript
 * import { removeAllActive } from '@choochmeque/tauri-plugin-notifications-api';
 * await removeAllActive()
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 */
async function removeAllActive(): Promise<void> {
  await invoke("plugin:notifications|remove_all");
}

/**
 * Creates a notification channel.
 *
 * @example
 * ```typescript
 * import { createChannel, Importance, Visibility } from '@choochmeque/tauri-plugin-notifications-api';
 * await createChannel({
 *   id: 'new-messages',
 *   name: 'New Messages',
 *   lights: true,
 *   vibration: true,
 *   importance: Importance.Default,
 *   visibility: Visibility.Private
 * });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 */
async function createChannel(channel: Channel): Promise<void> {
  await invoke("plugin:notifications|create_channel", { channel });
}

/**
 * Removes the channel with the given identifier.
 *
 * @example
 * ```typescript
 * import { removeChannel } from '@choochmeque/tauri-plugin-notifications-api';
 * await removeChannel('new-messages');
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 */
async function removeChannel(id: string): Promise<void> {
  await invoke("plugin:notifications|delete_channel", { id });
}

/**
 * Retrieves the list of notification channels.
 *
 * @example
 * ```typescript
 * import { channels } from '@choochmeque/tauri-plugin-notifications-api';
 * const notificationChannels = await channels();
 * ```
 *
 * @returns A promise resolving to the list of notification channels.
 */
async function channels(): Promise<Channel[]> {
  return await invoke("plugin:notifications|list_channels");
}

/**
 * Registers a listener for incoming notifications.
 *
 * @example
 * ```typescript
 * import { onNotificationReceived } from '@choochmeque/tauri-plugin-notifications-api';
 * const unlisten = await onNotificationReceived((notification) => {
 *   console.log('Notification received:', notification);
 * });
 * // Later, to stop listening:
 * // unlisten();
 * ```
 *
 * @param cb - Callback function to handle received notifications.
 * @returns A promise resolving to a function that removes the listener.
 */
async function onNotificationReceived(
  cb: (notification: Options) => void,
): Promise<PluginListener> {
  return await addPluginListener("notifications", "notification", cb);
}

/**
 * Registers a listener for notification action events.
 *
 * @example
 * ```typescript
 * import { onAction } from '@choochmeque/tauri-plugin-notifications-api';
 * const unlisten = await onAction((notification) => {
 *   console.log('Action performed on notification:', notification);
 * });
 * // Later, to stop listening:
 * // unlisten();
 * ```
 *
 * @param cb - Callback function to handle notification action results.
 * @returns A promise resolving to a function that removes the listener.
 */
async function onAction(
  cb: (action: ActionPerformedData) => void,
): Promise<PluginListener> {
  const listener = await addPluginListener(
    "notifications",
    "actionPerformed",
    cb,
  );
  try {
    await withActionListenerLock(async () => {
      if (actionListenerCount === 0) {
        await invoke("plugin:notifications|set_action_listener_active", {
          active: true,
        });
      }
      actionListenerCount += 1;
    });
  } catch (error) {
    await listener.unregister();
    throw error;
  }

  let unregistered = false;
  return {
    unregister: async () => {
      if (unregistered) return;
      await withActionListenerLock(async () => {
        if (actionListenerCount > 1) {
          actionListenerCount -= 1;
          return;
        }
        // Keep the final listener until native deactivation succeeds.
        await invoke("plugin:notifications|set_action_listener_active", {
          active: false,
        });
        actionListenerCount = 0;
      });
      unregistered = true;
      await listener.unregister();
    },
  } as PluginListener;
}

let actionListenerCount = 0;
let actionListenerLock = Promise.resolve();

function withActionListenerLock<T>(operation: () => Promise<T>): Promise<T> {
  const result = actionListenerLock.then(operation, operation);
  actionListenerLock = result.then(
    () => undefined,
    () => undefined,
  );
  return result;
}

/**
 * Data received when a notification is clicked/tapped.
 */
interface NotificationClickedData {
  /** Notification ID */
  id: number;
  /** Custom data payload attached to the notification */
  data?: Record<string, string>;
}

/**
 * Registers a listener for notification click/tap events.
 * This fires when the user taps on a notification (both push and local).
 *
 * This function handles cold-start scenarios where the app is launched by
 * tapping a notification. Any pending notification click data is automatically
 * delivered when the listener is registered.
 *
 * @example
 * ```typescript
 * import { onNotificationClicked } from '@choochmeque/tauri-plugin-notifications-api';
 * const unlisten = await onNotificationClicked((data) => {
 *   console.log('Notification clicked, id:', data.id);
 *   console.log('Custom data:', data.data);
 * });
 * ```
 *
 * @param cb - Callback function to handle notification clicks.
 * @returns A promise resolving to a function that removes the listener.
 */
async function onNotificationClicked(
  cb: (data: NotificationClickedData) => void,
): Promise<PluginListener> {
  const listener = await addPluginListener(
    "notifications",
    "notificationClicked",
    cb,
  );

  // Tell native side listener is active (triggers pending if any)
  await invoke("plugin:notifications|set_click_listener_active", {
    active: true,
  });

  // Return wrapped listener that notifies native side on unregister
  return {
    unregister: async () => {
      await invoke("plugin:notifications|set_click_listener_active", {
        active: false,
      });
      return listener.unregister();
    },
  } as PluginListener;
}

export type {
  Attachment,
  Options,
  Action,
  ActionType,
  PendingNotification,
  ActiveNotification,
  Channel,
  ScheduleInterval,
  NotificationClickedData,
  ActionPerformedData,
  ActionNotification,
  PushRegistration,
  PushProvider,
};

export {
  Importance,
  Visibility,
  sendNotification,
  requestPermission,
  isPermissionGranted,
  registerForPushNotifications,
  unregisterForPushNotifications,
  listDistributors,
  setDistributor,
  setToken,
  registerActionTypes,
  pending,
  cancel,
  cancelAll,
  active,
  removeActive,
  removeAllActive,
  createChannel,
  removeChannel,
  channels,
  onNotificationReceived,
  onAction,
  onNotificationClicked,
  Schedule,
  ScheduleEvery,
};
