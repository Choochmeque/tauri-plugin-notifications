// Tauri command handlers must take owned values: `State<'_, _>` is the framework's
// preferred wrapper, and serde-deserialized payloads (Vec, String, ...) cannot be borrowed.
#![allow(clippy::needless_pass_by_value)]

use serde::Deserialize;
use tauri::{command, plugin::PermissionState, AppHandle, Runtime, State};

use crate::{NotificationData, Notifications, Result};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct NotificationIdentifier {
    pub id: i32,
    #[allow(dead_code)]
    pub tag: Option<String>,
}

#[command]
pub async fn is_permission_granted<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<Option<bool>> {
    let state = notification.permission_state().await?;
    match state {
        PermissionState::Granted => Ok(Some(true)),
        PermissionState::Denied => Ok(Some(false)),
        PermissionState::Prompt | PermissionState::PromptWithRationale => Ok(None),
    }
}

#[command]
pub async fn request_permission<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<PermissionState> {
    notification.request_permission().await
}

#[command]
pub async fn register_for_push_notifications<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<String> {
    notification.register_for_push_notifications().await
}

#[command]
pub async fn unregister_for_push_notifications<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<()> {
    #[cfg(all(desktop, target_os = "linux", feature = "push-notifications"))]
    {
        notification.unregister_for_push_notifications_async().await
    }
    #[cfg(not(all(desktop, target_os = "linux", feature = "push-notifications")))]
    {
        notification.unregister_for_push_notifications()
    }
}

#[cfg(all(desktop, target_os = "linux", feature = "push-notifications"))]
#[command]
pub async fn list_distributors<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<Vec<String>> {
    notification.list_distributors().await
}

#[cfg(all(desktop, target_os = "linux", feature = "push-notifications"))]
#[command]
pub async fn set_distributor<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    name: String,
) -> Result<()> {
    notification.set_distributor(name).await
}

#[cfg(all(desktop, target_os = "linux", feature = "push-notifications"))]
#[command]
pub async fn set_token<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    token: String,
) -> Result<()> {
    notification.set_token(token).await
}

#[command]
pub async fn notify<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    options: NotificationData,
) -> Result<()> {
    let mut builder = notification.builder();
    builder.data = options;
    builder.show().await
}

#[command]
pub async fn register_action_types<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    types: Vec<crate::ActionType>,
) -> Result<()> {
    notification.register_action_types(types)
}

#[command]
pub async fn get_pending<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<Vec<crate::PendingNotification>> {
    notification.pending().await
}

#[command]
pub async fn get_active<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<Vec<crate::ActiveNotification>> {
    notification.active().await
}

#[command]
pub fn set_click_listener_active<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    active: bool,
) -> Result<()> {
    notification.set_click_listener_active(active)
}

#[command]
pub fn remove_active<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    notifications: Vec<NotificationIdentifier>,
) -> Result<()> {
    let ids: Vec<i32> = notifications.into_iter().map(|n| n.id).collect();
    notification.remove_active(ids)
}

#[command]
pub fn cancel<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    notifications: Vec<i32>,
) -> Result<()> {
    notification.cancel(notifications)
}

#[command]
pub fn cancel_all<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<()> {
    notification.cancel_all()
}

#[command]
pub fn create_channel<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    channel: crate::Channel,
) -> Result<()> {
    notification.create_channel(channel)
}

#[command]
pub fn delete_channel<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    id: String,
) -> Result<()> {
    notification.delete_channel(id)
}

#[command]
pub fn list_channels<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<Vec<crate::Channel>> {
    notification.list_channels()
}
