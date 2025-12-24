//! Windows implementation for notifications plugin using native Windows Toast API.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PermissionState, PluginApi},
    AppHandle, Runtime,
};
use windows::core::HSTRING;
use windows::Data::Xml::Dom::XmlDocument;
use windows::Foundation::{DateTime, TypedEventHandler};
use windows::UI::Notifications::{
    NotificationSetting, ScheduledToastNotification, ToastActivatedEventArgs, ToastNotification,
    ToastNotificationManager, ToastNotifier,
};

use crate::error::{ErrorResponse, PluginInvokeError};
use crate::models::*;

// Enable `?` operator for windows::core::Error
impl From<windows::core::Error> for crate::Error {
    fn from(err: windows::core::Error) -> Self {
        crate::Error::from(PluginInvokeError::InvokeRejected(ErrorResponse {
            code: Some(format!("0x{:08X}", err.code().0)),
            message: Some(err.message().to_string()),
            data: (),
        }))
    }
}

/// Shared plugin state wrapped in Arc for thread-safe access.
#[derive(Debug)]
pub struct WindowsPlugin {
    notifier: ToastNotifier,
    action_types: RwLock<HashMap<String, ActionType>>,
    click_listener_active: RwLock<bool>,
}

impl WindowsPlugin {
    fn action_types(&self) -> crate::Result<HashMap<String, ActionType>> {
        Ok(self
            .action_types
            .read()
            .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))?
            .clone())
    }

    fn action_types_mut(
        &self,
    ) -> crate::Result<std::sync::RwLockWriteGuard<'_, HashMap<String, ActionType>>> {
        self.action_types
            .write()
            .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))
    }

    fn is_click_listener_active(&self) -> crate::Result<bool> {
        Ok(*self
            .click_listener_active
            .read()
            .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))?)
    }

    fn set_click_listener(&self, active: bool) -> crate::Result<()> {
        *self
            .click_listener_active
            .write()
            .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))? = active;
        Ok(())
    }
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Notifications<R>> {
    // Use app identifier as AUMID (Application User Model ID)
    let app_id = &app.config().identifier;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(app_id))?;

    let plugin = Arc::new(WindowsPlugin {
        notifier,
        action_types: RwLock::new(HashMap::new()),
        click_listener_active: RwLock::new(false),
    });

    Ok(Notifications {
        app: app.clone(),
        plugin,
    })
}

impl<R: Runtime> crate::NotificationsBuilder<R> {
    /// Build toast notification XML using DOM API (safer than string concatenation).
    fn build_toast_xml(
        &self,
        action_types: &HashMap<String, ActionType>,
    ) -> crate::Result<XmlDocument> {
        let doc = XmlDocument::new()?;

        // Create root <toast>
        let toast = doc.CreateElement(&HSTRING::from("toast"))?;
        doc.AppendChild(&toast)?;

        // Create <visual><binding template="ToastGeneric">
        let visual = doc.CreateElement(&HSTRING::from("visual"))?;
        let binding = doc.CreateElement(&HSTRING::from("binding"))?;
        binding.SetAttribute(&HSTRING::from("template"), &HSTRING::from("ToastGeneric"))?;

        // Add <text> elements for title/body
        if let Some(title) = &self.data.title {
            let text = doc.CreateElement(&HSTRING::from("text"))?;
            text.SetInnerText(&HSTRING::from(title.as_str()))?;
            binding.AppendChild(&text)?;
        }

        if let Some(body) = &self.data.body {
            let text = doc.CreateElement(&HSTRING::from("text"))?;
            text.SetInnerText(&HSTRING::from(body.as_str()))?;
            binding.AppendChild(&text)?;
        }

        if let Some(large_body) = &self.data.large_body {
            let text = doc.CreateElement(&HSTRING::from("text"))?;
            text.SetInnerText(&HSTRING::from(large_body.as_str()))?;
            binding.AppendChild(&text)?;
        }

        // Add icon if specified
        if let Some(icon) = &self.data.icon {
            let image = doc.CreateElement(&HSTRING::from("image"))?;
            image.SetAttribute(
                &HSTRING::from("placement"),
                &HSTRING::from("appLogoOverride"),
            )?;
            image.SetAttribute(&HSTRING::from("src"), &HSTRING::from(icon.as_str()))?;
            binding.AppendChild(&image)?;
        }

        visual.AppendChild(&binding)?;
        toast.AppendChild(&visual)?;

        // Add <actions> if action_type_id specified
        if let Some(action_type_id) = &self.data.action_type_id {
            if let Some(action_type) = action_types.get(action_type_id) {
                let actions = doc.CreateElement(&HSTRING::from("actions"))?;
                for action in action_type.actions() {
                    let action_el = doc.CreateElement(&HSTRING::from("action"))?;
                    action_el
                        .SetAttribute(&HSTRING::from("content"), &HSTRING::from(action.title()))?;
                    action_el
                        .SetAttribute(&HSTRING::from("arguments"), &HSTRING::from(action.id()))?;
                    let activation_type = if action.foreground() {
                        "foreground"
                    } else {
                        "background"
                    };
                    action_el.SetAttribute(
                        &HSTRING::from("activationType"),
                        &HSTRING::from(activation_type),
                    )?;
                    actions.AppendChild(&action_el)?;
                }
                toast.AppendChild(&actions)?;
            }
        }

        // Add <audio silent="true"/> if silent
        if self.data.silent {
            let audio = doc.CreateElement(&HSTRING::from("audio"))?;
            audio.SetAttribute(&HSTRING::from("silent"), &HSTRING::from("true"))?;
            toast.AppendChild(&audio)?;
        }

        Ok(doc)
    }

    pub async fn show(self) -> crate::Result<()> {
        let action_types = self.plugin.action_types()?;
        let toast_xml = self.build_toast_xml(&action_types)?;

        let tag = HSTRING::from(self.data.id.to_string());
        let group = self.data.group.as_ref().map(|g| HSTRING::from(g.as_str()));

        // Check if this is a scheduled notification
        if let Some(schedule) = &self.data.schedule {
            let delivery_time = Self::schedule_to_datetime(schedule)?;
            let scheduled = ScheduledToastNotification::CreateScheduledToastNotification(
                &toast_xml,
                delivery_time,
            )?;

            scheduled.SetTag(&tag)?;
            if let Some(g) = &group {
                scheduled.SetGroup(g)?;
            }

            self.plugin.notifier.AddToSchedule(&scheduled)?;
        } else {
            // Immediate notification
            let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
            toast.SetTag(&tag)?;
            if let Some(g) = &group {
                toast.SetGroup(g)?;
            }

            if self.plugin.is_click_listener_active()? {
                let notification_id = self.data.id;
                let extra_data = self.data.extra.clone();

                toast.Activated(&TypedEventHandler::new(
                    move |_, args: &Option<windows::core::IInspectable>| {
                        if let Some(args) = args {
                            if let Ok(activated) = args.cast::<ToastActivatedEventArgs>() {
                                let arguments = activated
                                    .Arguments()
                                    .map(|s| s.to_string_lossy())
                                    .unwrap_or_default();

                                if arguments.is_empty() {
                                    let payload = serde_json::json!({
                                        "id": notification_id,
                                        "data": extra_data,
                                    });
                                    let _ = crate::listeners::trigger(
                                        "notificationClicked",
                                        payload.to_string(),
                                    );
                                } else {
                                    let payload = serde_json::json!({
                                        "id": notification_id,
                                        "actionTypeId": arguments.to_string(),
                                    });
                                    let _ = crate::listeners::trigger(
                                        "actionPerformed",
                                        payload.to_string(),
                                    );
                                }
                            }
                        }
                        Ok(())
                    },
                ))?;
            }

            self.plugin.notifier.Show(&toast)?;
        }

        // Trigger notification event
        let payload = serde_json::json!({
            "id": self.data.id,
            "title": self.data.title,
            "body": self.data.body,
            "actionTypeId": self.data.action_type_id,
            "extra": self.data.extra,
        });
        let _ = crate::listeners::trigger("notification", payload.to_string());

        Ok(())
    }

    /// Convert Schedule to Windows DateTime.
    fn schedule_to_datetime(schedule: &Schedule) -> crate::Result<DateTime> {
        let now = time::OffsetDateTime::now_utc();

        let delivery_time = match schedule {
            Schedule::At { date, .. } => *date,
            Schedule::Interval { interval, .. } => {
                // Build duration from interval fields
                let seconds = interval.second.unwrap_or(0) as i64;
                let minutes = interval.minute.unwrap_or(0) as i64;
                let hours = interval.hour.unwrap_or(0) as i64;
                let days = interval.day.unwrap_or(0) as i64;
                let total_seconds = seconds + minutes * 60 + hours * 3600 + days * 86400;
                now + time::Duration::seconds(total_seconds)
            }
            Schedule::Every {
                interval, count, ..
            } => {
                let base_seconds: i64 = match interval {
                    ScheduleEvery::Year => 365 * 86400,
                    ScheduleEvery::Month => 30 * 86400,
                    ScheduleEvery::TwoWeeks => 14 * 86400,
                    ScheduleEvery::Week => 7 * 86400,
                    ScheduleEvery::Day => 86400,
                    ScheduleEvery::Hour => 3600,
                    ScheduleEvery::Minute => 60,
                    ScheduleEvery::Second => 1,
                };
                now + time::Duration::seconds(base_seconds * (*count as i64))
            }
        };

        // Convert to Windows DateTime (100-nanosecond intervals since 1601-01-01)
        // Unix epoch is 11644473600 seconds after Windows epoch
        let unix_nanos = delivery_time.unix_timestamp_nanos();
        let windows_ticks = (unix_nanos / 100) + 116_444_736_000_000_000i128;

        Ok(DateTime {
            UniversalTime: windows_ticks as i64,
        })
    }
}

pub struct Notifications<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    plugin: Arc<WindowsPlugin>,
}

impl<R: Runtime> Notifications<R> {
    pub fn builder(&self) -> crate::NotificationsBuilder<R> {
        crate::NotificationsBuilder::new(self.app.clone(), self.plugin.clone())
    }

    pub async fn request_permission(&self) -> crate::Result<PermissionState> {
        // Windows doesn't have a runtime permission prompt like mobile
        // We can only check the current state
        self.permission_state().await
    }

    pub async fn register_for_push_notifications(&self) -> crate::Result<String> {
        Err(crate::Error::Io(std::io::Error::other(
            "Push notifications are not supported on Windows",
        )))
    }

    pub fn unregister_for_push_notifications(&self) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Push notifications are not supported on Windows",
        )))
    }

    pub async fn permission_state(&self) -> crate::Result<PermissionState> {
        match self.plugin.notifier.Setting()? {
            NotificationSetting::Enabled => Ok(PermissionState::Granted),
            NotificationSetting::DisabledForApplication
            | NotificationSetting::DisabledForUser
            | NotificationSetting::DisabledByGroupPolicy
            | NotificationSetting::DisabledByManifest => Ok(PermissionState::Denied),
            _ => Ok(PermissionState::Prompt),
        }
    }

    pub fn register_action_types(&self, types: Vec<ActionType>) -> crate::Result<()> {
        let mut action_types = self.plugin.action_types_mut()?;
        for action_type in types {
            action_types.insert(action_type.id().to_string(), action_type);
        }
        Ok(())
    }

    pub fn remove_active(&self, notifications: Vec<i32>) -> crate::Result<()> {
        let history = ToastNotificationManager::History()?;
        for id in notifications {
            // Remove by tag (which we set to the notification ID)
            let _ = history.Remove(&HSTRING::from(id.to_string()));
        }
        Ok(())
    }

    pub async fn active(&self) -> crate::Result<Vec<ActiveNotification>> {
        let history = ToastNotificationManager::History()?;
        let app_id = &self.app.config().identifier;
        let notifications = history.GetHistoryWithId(&HSTRING::from(app_id))?;

        let mut result = Vec::new();
        for i in 0..notifications.Size()? {
            let notification = notifications.GetAt(i)?;
            let tag = notification.Tag()?.to_string_lossy();
            let id = tag.parse::<i32>().unwrap_or(0);
            let group = notification.Group().ok().map(|s| s.to_string_lossy());

            // Extract title/body from XML content
            let (title, body) = if let Ok(content) = notification.Content() {
                let text_elements = content.GetElementsByTagName(&HSTRING::from("text"))?;
                let title = text_elements
                    .GetAt(0)
                    .ok()
                    .and_then(|el| el.InnerText().ok())
                    .map(|s| s.to_string_lossy());
                let body = text_elements
                    .GetAt(1)
                    .ok()
                    .and_then(|el| el.InnerText().ok())
                    .map(|s| s.to_string_lossy());
                (title, body)
            } else {
                (None, None)
            };

            result.push(ActiveNotification {
                id,
                tag: Some(tag),
                title,
                body,
                group,
                group_summary: false,
                data: HashMap::new(),
                extra: HashMap::new(),
                attachments: Vec::new(),
                action_type_id: None,
                schedule: None,
                sound: None,
            });
        }

        Ok(result)
    }

    pub fn remove_all_active(&self) -> crate::Result<()> {
        let history = ToastNotificationManager::History()?;
        let app_id = &self.app.config().identifier;
        history.ClearWithId(&HSTRING::from(app_id))?;
        Ok(())
    }

    pub async fn pending(&self) -> crate::Result<Vec<PendingNotification>> {
        let scheduled = self.plugin.notifier.GetScheduledToastNotifications()?;
        let mut result = Vec::new();

        for i in 0..scheduled.Size()? {
            let notification = scheduled.GetAt(i)?;
            let tag = notification.Tag()?.to_string_lossy();
            let id = tag.parse::<i32>().unwrap_or(0);

            let (title, body) = if let Ok(content) = notification.Content() {
                let text_elements = content.GetElementsByTagName(&HSTRING::from("text"))?;
                let title = text_elements
                    .GetAt(0)
                    .ok()
                    .and_then(|el| el.InnerText().ok())
                    .map(|s| s.to_string_lossy());
                let body = text_elements
                    .GetAt(1)
                    .ok()
                    .and_then(|el| el.InnerText().ok())
                    .map(|s| s.to_string_lossy());
                (title, body)
            } else {
                (None, None)
            };

            // Convert Windows DateTime back to Schedule::At
            let schedule = notification.DeliveryTime().ok().and_then(|dt| {
                let windows_ticks = dt.UniversalTime;
                let unix_nanos = (windows_ticks as i128 - 116_444_736_000_000_000i128) * 100;
                time::OffsetDateTime::from_unix_timestamp_nanos(unix_nanos)
                    .ok()
                    .map(|date| Schedule::At {
                        date,
                        repeating: false,
                        allow_while_idle: false,
                    })
            });

            // PendingNotification requires schedule (not Option), skip if we can't extract it
            if let Some(schedule) = schedule {
                result.push(PendingNotification {
                    id,
                    title,
                    body,
                    schedule,
                });
            }
        }

        Ok(result)
    }

    pub fn cancel(&self, notifications: Vec<i32>) -> crate::Result<()> {
        let scheduled = self.plugin.notifier.GetScheduledToastNotifications()?;
        let ids_to_cancel: std::collections::HashSet<_> = notifications.into_iter().collect();

        for i in 0..scheduled.Size()? {
            if let Ok(notification) = scheduled.GetAt(i) {
                if let Ok(tag) = notification.Tag() {
                    if let Ok(id) = tag.to_string_lossy().parse::<i32>() {
                        if ids_to_cancel.contains(&id) {
                            let _ = self.plugin.notifier.RemoveFromSchedule(&notification);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn cancel_all(&self) -> crate::Result<()> {
        let scheduled = self.plugin.notifier.GetScheduledToastNotifications()?;
        for i in 0..scheduled.Size()? {
            if let Ok(notification) = scheduled.GetAt(i) {
                let _ = self.plugin.notifier.RemoveFromSchedule(&notification);
            }
        }
        Ok(())
    }

    pub fn set_click_listener_active(&self, active: bool) -> crate::Result<()> {
        self.plugin.set_click_listener(active)
    }

    /// Create a notification channel (not supported on Windows).
    pub fn create_channel(&self, _channel: crate::Channel) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported on Windows",
        )))
    }

    /// Delete a notification channel (not supported on Windows).
    pub fn delete_channel(&self, _id: impl Into<String>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported on Windows",
        )))
    }

    /// List notification channels (not supported on Windows).
    pub fn list_channels(&self) -> crate::Result<Vec<crate::Channel>> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported on Windows",
        )))
    }
}
