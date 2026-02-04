//! Send message notifications (brief auto-expiring OS window element) to your user. Can also be used with the Notification Web API.

use serde::Serialize;
#[cfg(mobile)]
use tauri::plugin::PluginHandle;
#[cfg(desktop)]
use tauri::AppHandle;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;
pub use tauri::plugin::PermissionState;

#[cfg(all(desktop, feature = "notify-rust"))]
mod desktop;
#[cfg(all(target_os = "macos", not(feature = "notify-rust")))]
mod macos;
#[cfg(mobile)]
mod mobile;
#[cfg(all(target_os = "windows", not(feature = "notify-rust")))]
mod windows;

mod commands;
mod error;
#[cfg(desktop)]
mod listeners;
mod models;

pub use error::{Error, Result};

#[cfg(all(desktop, feature = "notify-rust"))]
pub use desktop::Notifications;
#[cfg(all(target_os = "macos", not(feature = "notify-rust")))]
pub use macos::Notifications;
#[cfg(mobile)]
pub use mobile::Notifications;
#[cfg(all(target_os = "windows", not(feature = "notify-rust")))]
pub use windows::Notifications;

/// The notification builder.
#[derive(Debug)]
pub struct NotificationsBuilder<R: Runtime> {
    #[cfg(desktop)]
    #[allow(dead_code)]
    app: AppHandle<R>,
    #[cfg(all(target_os = "macos", not(feature = "notify-rust")))]
    plugin: std::sync::Arc<macos::NotificationPlugin>,
    #[cfg(all(target_os = "windows", not(feature = "notify-rust")))]
    plugin: std::sync::Arc<windows::WindowsPlugin>,
    #[cfg(mobile)]
    handle: PluginHandle<R>,
    pub(crate) data: NotificationData,
}

impl<R: Runtime> NotificationsBuilder<R> {
    #[cfg(all(desktop, feature = "notify-rust"))]
    fn new(app: AppHandle<R>) -> Self {
        Self {
            app,
            data: Default::default(),
        }
    }

    #[cfg(all(target_os = "macos", not(feature = "notify-rust")))]
    fn new(app: AppHandle<R>, plugin: std::sync::Arc<macos::NotificationPlugin>) -> Self {
        Self {
            app,
            plugin,
            data: Default::default(),
        }
    }

    #[cfg(all(target_os = "windows", not(feature = "notify-rust")))]
    fn new(app: AppHandle<R>, plugin: std::sync::Arc<windows::WindowsPlugin>) -> Self {
        Self {
            app,
            plugin,
            data: Default::default(),
        }
    }

    #[cfg(mobile)]
    fn new(handle: PluginHandle<R>) -> Self {
        Self {
            handle,
            data: Default::default(),
        }
    }

    /// Sets the notification identifier.
    pub fn id(mut self, id: i32) -> Self {
        self.data.id = id;
        self
    }

    /// Identifier of the {@link Channel} that delivers this notification.
    ///
    /// If the channel does not exist, the notification won't fire.
    /// Make sure the channel exists with {@link listChannels} and {@link createChannel}.
    pub fn channel_id(mut self, id: impl Into<String>) -> Self {
        self.data.channel_id.replace(id.into());
        self
    }

    /// Sets the notification title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.data.title.replace(title.into());
        self
    }

    /// Sets the notification body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.data.body.replace(body.into());
        self
    }

    /// Schedule this notification to fire on a later time or a fixed interval.
    pub fn schedule(mut self, schedule: Schedule) -> Self {
        self.data.schedule.replace(schedule);
        self
    }

    /// Multiline text.
    /// Changes the notification style to big text.
    /// Cannot be used with `inboxLines`.
    pub fn large_body(mut self, large_body: impl Into<String>) -> Self {
        self.data.large_body.replace(large_body.into());
        self
    }

    /// Detail text for the notification with `largeBody`, `inboxLines` or `groupSummary`.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.data.summary.replace(summary.into());
        self
    }

    /// Defines an action type for this notification.
    pub fn action_type_id(mut self, action_type_id: impl Into<String>) -> Self {
        self.data.action_type_id.replace(action_type_id.into());
        self
    }

    /// Identifier used to group multiple notifications.
    ///
    /// <https://developer.apple.com/documentation/usernotifications/unmutablenotificationcontent/1649872-threadidentifier>
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.data.group.replace(group.into());
        self
    }

    /// Instructs the system that this notification is the summary of a group on Android.
    pub fn group_summary(mut self) -> Self {
        self.data.group_summary = true;
        self
    }

    /// The sound resource name. Only available on mobile.
    pub fn sound(mut self, sound: impl Into<String>) -> Self {
        self.data.sound.replace(sound.into());
        self
    }

    /// Append an inbox line to the notification.
    /// Changes the notification style to inbox.
    /// Cannot be used with `largeBody`.
    ///
    /// Only supports up to 5 lines.
    pub fn inbox_line(mut self, line: impl Into<String>) -> Self {
        self.data.inbox_lines.push(line.into());
        self
    }

    /// Notification icon.
    ///
    /// On Android the icon must be placed in the app's `res/drawable` folder.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.data.icon.replace(icon.into());
        self
    }

    /// Notification large icon (Android).
    ///
    /// The icon must be placed in the app's `res/drawable` folder.
    pub fn large_icon(mut self, large_icon: impl Into<String>) -> Self {
        self.data.large_icon.replace(large_icon.into());
        self
    }

    /// Icon color on Android.
    pub fn icon_color(mut self, icon_color: impl Into<String>) -> Self {
        self.data.icon_color.replace(icon_color.into());
        self
    }

    /// Append an attachment to the notification.
    pub fn attachment(mut self, attachment: Attachment) -> Self {
        self.data.attachments.push(attachment);
        self
    }

    /// Adds an extra payload to store in the notification.
    pub fn extra(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(value) = serde_json::to_value(value) {
            self.data.extra.insert(key.into(), value);
        }
        self
    }

    /// If true, the notification cannot be dismissed by the user on Android.
    ///
    /// An application service must manage the dismissal of the notification.
    /// It is typically used to indicate a background task that is pending (e.g. a file download)
    /// or the user is engaged with (e.g. playing music).
    pub fn ongoing(mut self) -> Self {
        self.data.ongoing = true;
        self
    }

    /// Automatically cancel the notification when the user clicks on it.
    pub fn auto_cancel(mut self) -> Self {
        self.data.auto_cancel = true;
        self
    }

    /// Changes the notification presentation to be silent on iOS (no badge, no sound, not listed).
    pub fn silent(mut self) -> Self {
        self.data.silent = true;
        self
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the notification APIs.
pub trait NotificationsExt<R: Runtime> {
    fn notifications(&self) -> &Notifications<R>;
}

impl<R: Runtime, T: Manager<R>> crate::NotificationsExt<R> for T {
    fn notifications(&self) -> &Notifications<R> {
        self.state::<Notifications<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("notifications")
        .invoke_handler(tauri::generate_handler![
            commands::notify,
            commands::request_permission,
            commands::register_for_push_notifications,
            commands::unregister_for_push_notifications,
            commands::is_permission_granted,
            commands::register_action_types,
            commands::get_pending,
            commands::get_active,
            commands::set_click_listener_active,
            commands::remove_active,
            commands::remove_all,
            commands::cancel,
            commands::cancel_all,
            commands::create_channel,
            commands::delete_channel,
            commands::list_channels,
            #[cfg(desktop)]
            listeners::register_listener,
            #[cfg(desktop)]
            listeners::remove_listener,
        ])
        .setup(|app, api| {
            #[cfg(desktop)]
            listeners::init();
            #[cfg(mobile)]
            let notification = mobile::init(app, api)?;
            #[cfg(all(desktop, feature = "notify-rust"))]
            let notification = desktop::init(app, api)?;
            #[cfg(all(target_os = "macos", not(feature = "notify-rust")))]
            let notification = macos::init(app, api)?;
            #[cfg(all(target_os = "windows", not(feature = "notify-rust")))]
            let notification = windows::init(app, api)?;
            app.manage(notification);
            Ok(())
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test builder without needing a runtime
    #[cfg(desktop)]
    fn create_test_data() -> NotificationData {
        NotificationData::default()
    }

    #[cfg(mobile)]
    fn create_test_data() -> NotificationData {
        NotificationData::default()
    }

    #[test]
    fn test_notification_data_id() {
        let mut data = create_test_data();
        data.id = 42;
        assert_eq!(data.id, 42);
    }

    #[test]
    fn test_notification_data_channel_id() {
        let mut data = create_test_data();
        data.channel_id = Some("test_channel".to_string());
        assert_eq!(data.channel_id, Some("test_channel".to_string()));
    }

    #[test]
    fn test_notification_data_title() {
        let mut data = create_test_data();
        data.title = Some("Test Title".to_string());
        assert_eq!(data.title, Some("Test Title".to_string()));
    }

    #[test]
    fn test_notification_data_body() {
        let mut data = create_test_data();
        data.body = Some("Test Body".to_string());
        assert_eq!(data.body, Some("Test Body".to_string()));
    }

    #[test]
    fn test_notification_data_large_body() {
        let mut data = create_test_data();
        data.large_body = Some("Large Body Text".to_string());
        assert_eq!(data.large_body, Some("Large Body Text".to_string()));
    }

    #[test]
    fn test_notification_data_summary() {
        let mut data = create_test_data();
        data.summary = Some("Summary Text".to_string());
        assert_eq!(data.summary, Some("Summary Text".to_string()));
    }

    #[test]
    fn test_notification_data_action_type_id() {
        let mut data = create_test_data();
        data.action_type_id = Some("action_type".to_string());
        assert_eq!(data.action_type_id, Some("action_type".to_string()));
    }

    #[test]
    fn test_notification_data_group() {
        let mut data = create_test_data();
        data.group = Some("test_group".to_string());
        assert_eq!(data.group, Some("test_group".to_string()));
    }

    #[test]
    fn test_notification_data_group_summary() {
        let mut data = create_test_data();
        data.group_summary = true;
        assert!(data.group_summary);
    }

    #[test]
    fn test_notification_data_sound() {
        let mut data = create_test_data();
        data.sound = Some("notification_sound".to_string());
        assert_eq!(data.sound, Some("notification_sound".to_string()));
    }

    #[test]
    fn test_notification_data_inbox_lines() {
        let mut data = create_test_data();
        data.inbox_lines.push("Line 1".to_string());
        data.inbox_lines.push("Line 2".to_string());
        assert_eq!(data.inbox_lines.len(), 2);
        assert_eq!(data.inbox_lines[0], "Line 1");
        assert_eq!(data.inbox_lines[1], "Line 2");
    }

    #[test]
    fn test_notification_data_icon() {
        let mut data = create_test_data();
        data.icon = Some("icon_name".to_string());
        assert_eq!(data.icon, Some("icon_name".to_string()));
    }

    #[test]
    fn test_notification_data_large_icon() {
        let mut data = create_test_data();
        data.large_icon = Some("large_icon_name".to_string());
        assert_eq!(data.large_icon, Some("large_icon_name".to_string()));
    }

    #[test]
    fn test_notification_data_icon_color() {
        let mut data = create_test_data();
        data.icon_color = Some("#FF0000".to_string());
        assert_eq!(data.icon_color, Some("#FF0000".to_string()));
    }

    #[test]
    fn test_notification_data_attachments() {
        let mut data = create_test_data();
        let url = url::Url::parse("https://example.com/image.png").expect("Failed to parse URL");
        let attachment = Attachment::new("attachment1", url);
        data.attachments.push(attachment);
        assert_eq!(data.attachments.len(), 1);
    }

    #[test]
    fn test_notification_data_extra() {
        let mut data = create_test_data();
        data.extra
            .insert("key1".to_string(), serde_json::json!("value1"));
        data.extra.insert("key2".to_string(), serde_json::json!(42));
        assert_eq!(data.extra.len(), 2);
        assert_eq!(data.extra.get("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(data.extra.get("key2"), Some(&serde_json::json!(42)));
    }

    #[test]
    fn test_notification_data_ongoing() {
        let mut data = create_test_data();
        data.ongoing = true;
        assert!(data.ongoing);
    }

    #[test]
    fn test_notification_data_auto_cancel() {
        let mut data = create_test_data();
        data.auto_cancel = true;
        assert!(data.auto_cancel);
    }

    #[test]
    fn test_notification_data_silent() {
        let mut data = create_test_data();
        data.silent = true;
        assert!(data.silent);
    }

    #[test]
    fn test_notification_data_schedule() {
        let mut data = create_test_data();
        let schedule = Schedule::Every {
            interval: ScheduleEvery::Day,
            count: 1,
            allow_while_idle: false,
        };
        data.schedule = Some(schedule);
        assert!(data.schedule.is_some());
        assert!(matches!(data.schedule, Some(Schedule::Every { .. })));
    }
}
