use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PermissionState, PluginApi},
    AppHandle, Runtime,
};

use crate::models::*;

use std::{collections::HashMap, sync::Arc};

pub use ffi::NotificationPlugin;

#[swift_bridge::bridge]
mod ffi {
    pub enum FFIResult {
        Err(String), // error message from Swift
    }

    extern "Rust" {
        #[swift_bridge(swift_name = "bridgeTrigger")]
        fn bridge_trigger(event: String, payload: String) -> Result<(), FFIResult>;
    }

    extern "Swift" {
        #[swift_bridge(Sendable)]
        type NotificationPlugin;
        #[swift_bridge(init, swift_name = "initPlugin")]
        fn init_plugin() -> NotificationPlugin;

        async fn show(&self, args: String) -> Result<i32, FFIResult>;

        async fn requestPermissions(&self) -> Result<String, FFIResult>;
        fn registerForPushNotifications(&self) -> Result<String, FFIResult>;
        fn unregisterForPushNotifications(&self) -> Result<String, FFIResult>;
        async fn checkPermissions(&self) -> Result<String, FFIResult>;
        fn cancel(&self, args: String) -> Result<(), FFIResult>;
        fn cancelAll(&self) -> Result<(), FFIResult>;
        async fn getPending(&self) -> Result<String, FFIResult>;
        fn registerActionTypes(&self, args: String) -> Result<(), FFIResult>;
        fn removeActive(&self, args: String) -> Result<(), FFIResult>;
        fn removeAllActive(&self) -> Result<(), FFIResult>;
        async fn getActive(&self) -> Result<String, FFIResult>;
        fn setClickListenerActive(&self, args: String) -> Result<(), FFIResult>;
    }
}

/// Extension trait for parsing FFI responses from Swift into typed Rust results.
trait ParseFfiResponse {
    /// Deserializes a JSON response into the target type, converting FFI errors
    /// into plugin errors.
    fn parse<T: DeserializeOwned>(self) -> crate::Result<T>;
}

impl ParseFfiResponse for Result<String, ffi::FFIResult> {
    fn parse<T: DeserializeOwned>(self) -> crate::Result<T> {
        match self {
            Ok(json) => serde_json::from_str(&json)
                .map_err(|e| crate::error::PluginInvokeError::CannotDeserializeResponse(e).into()),
            Err(ffi::FFIResult::Err(msg)) => Err(crate::error::PluginInvokeError::InvokeRejected(
                crate::error::ErrorResponse {
                    code: None,
                    message: Some(msg),
                    data: (),
                },
            )
            .into()),
        }
    }
}

trait ParseFfiVoidResponse {
    fn parse_void(self) -> crate::Result<()>;
}

impl ParseFfiVoidResponse for Result<(), ffi::FFIResult> {
    fn parse_void(self) -> crate::Result<()> {
        match self {
            Ok(()) => Ok(()),
            Err(ffi::FFIResult::Err(msg)) => Err(crate::error::PluginInvokeError::InvokeRejected(
                crate::error::ErrorResponse {
                    code: None,
                    message: Some(msg),
                    data: (),
                },
            )
            .into()),
        }
    }
}

impl ParseFfiVoidResponse for Result<i32, ffi::FFIResult> {
    fn parse_void(self) -> crate::Result<()> {
        match self {
            Ok(_) => Ok(()),
            Err(ffi::FFIResult::Err(msg)) => Err(crate::error::PluginInvokeError::InvokeRejected(
                crate::error::ErrorResponse {
                    code: None,
                    message: Some(msg),
                    data: (),
                },
            )
            .into()),
        }
    }
}

/// Called by Swift via FFI when transaction updates occur.
fn bridge_trigger(event: String, payload: String) -> Result<(), ffi::FFIResult> {
    crate::listeners::trigger(&event, payload)
        .map_err(|e| ffi::FFIResult::Err(format!("Failed to trigger event '{event}': {e}")))
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Notifications<R>> {
    Ok(Notifications {
        app: app.clone(),
        plugin: Arc::new(ffi::NotificationPlugin::init_plugin()),
    })
}

impl<R: Runtime> crate::NotificationsBuilder<R> {
    pub async fn show(self) -> crate::Result<()> {
        self.plugin
            .show(
                serde_json::to_string(&self.data)
                    .map_err(|e| crate::error::PluginInvokeError::CannotSerializePayload(e))?,
            )
            .await
            .parse_void()
    }
}

pub struct Notifications<R: Runtime> {
    app: AppHandle<R>,
    plugin: Arc<ffi::NotificationPlugin>,
}

impl<R: Runtime> Notifications<R> {
    pub fn builder(&self) -> crate::NotificationsBuilder<R> {
        crate::NotificationsBuilder::new(self.app.clone(), self.plugin.clone())
    }

    pub async fn request_permission(&self) -> crate::Result<PermissionState> {
        self.plugin.requestPermissions().await.parse()
    }

    pub fn register_for_push_notifications(&self) -> crate::Result<String> {
        #[cfg(feature = "push-notifications")]
        {
            self.plugin.registerForPushNotifications().parse()
        }
        #[cfg(not(feature = "push-notifications"))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Push notifications feature is not enabled",
            )))
        }
    }

    pub fn unregister_for_push_notifications(&self) -> crate::Result<()> {
        #[cfg(feature = "push-notifications")]
        {
            self.plugin.unregisterForPushNotifications().parse()
        }
        #[cfg(not(feature = "push-notifications"))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Push notifications feature is not enabled",
            )))
        }
    }

    pub async fn permission_state(&self) -> crate::Result<PermissionState> {
        self.plugin.checkPermissions().await.parse()
    }

    pub fn register_action_types(&self, types: Vec<ActionType>) -> crate::Result<()> {
        let mut args = HashMap::new();
        args.insert("types", types);
        self.plugin
            .registerActionTypes(
                serde_json::to_string(&args)
                    .map_err(|e| crate::error::PluginInvokeError::CannotSerializePayload(e))?,
            )
            .parse_void()
    }

    pub fn remove_active(&self, notifications: Vec<i32>) -> crate::Result<()> {
        let mut args = HashMap::new();
        args.insert(
            "notifications",
            notifications
                .into_iter()
                .map(|id| {
                    let mut notification = HashMap::new();
                    notification.insert("id", id);
                    notification
                })
                .collect::<Vec<HashMap<&str, i32>>>(),
        );
        self.plugin
            .removeActive(
                serde_json::to_string(&args)
                    .map_err(|e| crate::error::PluginInvokeError::CannotSerializePayload(e))?,
            )
            .parse_void()
    }

    pub async fn active(&self) -> crate::Result<Vec<ActiveNotification>> {
        self.plugin.getActive().await.parse()
    }

    pub fn remove_all_active(&self) -> crate::Result<()> {
        self.plugin.removeAllActive().parse_void()
    }

    pub async fn pending(&self) -> crate::Result<Vec<PendingNotification>> {
        self.plugin.getPending().await.parse()
    }

    /// Cancel pending notifications.
    pub fn cancel(&self, notifications: Vec<i32>) -> crate::Result<()> {
        let mut args = HashMap::new();
        args.insert("notifications", notifications);
        self.plugin
            .cancel(
                serde_json::to_string(&args)
                    .map_err(|e| crate::error::PluginInvokeError::CannotSerializePayload(e))?,
            )
            .parse_void()
    }

    /// Cancel all pending notifications.
    pub fn cancel_all(&self) -> crate::Result<()> {
        self.plugin.cancelAll().parse_void()
    }

    /// Set click listener active state.
    /// Used internally to track if JS listener is registered.
    pub fn set_click_listener_active(&self, active: bool) -> crate::Result<()> {
        let mut args = HashMap::new();
        args.insert("active", active);
        self.plugin
            .setClickListenerActive(
                serde_json::to_string(&args)
                    .map_err(|e| crate::error::PluginInvokeError::CannotSerializePayload(e))?,
            )
            .parse_void()
    }
}
