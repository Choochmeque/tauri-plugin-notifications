// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
    _notification: State<'_, Notifications<R>>,
) -> Result<String> {
    #[cfg(feature = "push-notifications")]
    {
        _notification.register_for_push_notifications()
    }
    #[cfg(not(feature = "push-notifications"))]
    {
        Err(crate::Error::Io(std::io::Error::other(
            "Push notifications feature is not enabled",
        )))
    }
}

#[command]
pub(crate) async fn unregister_for_push_notifications<R: Runtime>(
    _app: AppHandle<R>,
    _notification: State<'_, Notifications<R>>,
) -> Result<()> {
    #[cfg(feature = "push-notifications")]
    {
        _notification.unregister_for_push_notifications()
    }
    #[cfg(not(feature = "push-notifications"))]
    {
        Err(crate::Error::Io(std::io::Error::other(
            "Push notifications feature is not enabled",
        )))
    }
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
