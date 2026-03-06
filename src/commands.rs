use serde::Deserialize;
use tauri::{command, plugin::PermissionState, AppHandle, Runtime, State};

use crate::{NotificationData, Notifications, Result};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct NotificationIdentifier {
    pub id: i32,
    #[allow(dead_code)]
    pub tag: Option<String>,
}

#[command]
pub(crate) async fn is_permission_granted<R: Runtime>(
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
pub(crate) async fn request_permission<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<PermissionState> {
    notification.request_permission().await
}

#[command]
pub(crate) async fn register_for_push_notifications<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<String> {
    notification.register_for_push_notifications().await
}

#[command]
pub(crate) async fn unregister_for_push_notifications<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<()> {
    notification.unregister_for_push_notifications()
}

#[command]
pub(crate) async fn notify<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    options: NotificationData,
) -> Result<()> {
    let mut builder = notification.builder();
    builder.data = options;
    builder.show().await
}

#[command]
pub(crate) async fn register_action_types<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    types: Vec<crate::ActionType>,
) -> Result<()> {
    notification.register_action_types(types)
}

#[command]
pub(crate) async fn get_pending<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<Vec<crate::PendingNotification>> {
    notification.pending().await
}

#[command]
pub(crate) async fn get_active<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<Vec<crate::ActiveNotification>> {
    notification.active().await
}

#[command]
pub(crate) fn set_click_listener_active<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    active: bool,
) -> Result<()> {
    notification.set_click_listener_active(active)
}

#[command]
pub(crate) fn remove_active<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    notifications: Vec<NotificationIdentifier>,
) -> Result<()> {
    let ids: Vec<i32> = notifications.into_iter().map(|n| n.id).collect();
    notification.remove_active(ids)
}

#[command]
pub(crate) fn remove_all<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<()> {
    notification.remove_all_active()
}

#[command]
pub(crate) fn cancel<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    notifications: Vec<i32>,
) -> Result<()> {
    notification.cancel(notifications)
}

#[command]
pub(crate) fn cancel_all<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<()> {
    notification.cancel_all()
}

#[command]
pub(crate) fn create_channel<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    channel: crate::Channel,
) -> Result<()> {
    notification.create_channel(channel)
}

#[command]
pub(crate) fn delete_channel<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
    id: String,
) -> Result<()> {
    notification.delete_channel(id)
}

#[command]
pub(crate) fn list_channels<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notifications<R>>,
) -> Result<Vec<crate::Channel>> {
    notification.list_channels()
}
