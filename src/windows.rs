//! Windows implementation for notifications plugin.

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PermissionState, PluginApi},
    AppHandle, Runtime,
};

use crate::models::*;

fn not_supported(feature: &str) -> crate::Error {
    crate::Error::Io(std::io::Error::other(format!(
        "{} is not supported on Windows without notify-rust feature",
        feature
    )))
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Notifications<R>> {
    Ok(Notifications { app: app.clone() })
}

impl<R: Runtime> crate::NotificationsBuilder<R> {
    pub async fn show(self) -> crate::Result<()> {
        Err(not_supported("Notifications"))
    }
}

pub struct Notifications<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> Notifications<R> {
    pub fn builder(&self) -> crate::NotificationsBuilder<R> {
        crate::NotificationsBuilder::new(self.app.clone())
    }

    pub async fn request_permission(&self) -> crate::Result<PermissionState> {
        // Windows doesn't require permission for notifications
        Ok(PermissionState::Granted)
    }

    pub async fn register_for_push_notifications(&self) -> crate::Result<String> {
        Err(not_supported("Push notifications"))
    }

    pub fn unregister_for_push_notifications(&self) -> crate::Result<()> {
        Err(not_supported("Push notifications"))
    }

    pub async fn permission_state(&self) -> crate::Result<PermissionState> {
        // Windows doesn't require permission for notifications
        Ok(PermissionState::Granted)
    }

    pub fn register_action_types(&self, _types: Vec<ActionType>) -> crate::Result<()> {
        Err(not_supported("Action types"))
    }

    pub fn remove_active(&self, _notifications: Vec<i32>) -> crate::Result<()> {
        Err(not_supported("Remove active notifications"))
    }

    pub async fn active(&self) -> crate::Result<Vec<ActiveNotification>> {
        Err(not_supported("Active notifications"))
    }

    pub fn remove_all_active(&self) -> crate::Result<()> {
        Err(not_supported("Remove active notifications"))
    }

    pub async fn pending(&self) -> crate::Result<Vec<PendingNotification>> {
        Err(not_supported("Pending notifications"))
    }

    pub fn cancel(&self, _notifications: Vec<i32>) -> crate::Result<()> {
        Err(not_supported("Cancel notifications"))
    }

    pub fn cancel_all(&self) -> crate::Result<()> {
        Err(not_supported("Cancel notifications"))
    }

    pub fn set_click_listener_active(&self, _active: bool) -> crate::Result<()> {
        Err(not_supported("Click listener"))
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
