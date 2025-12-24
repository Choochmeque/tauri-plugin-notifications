use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PermissionState, PluginApi},
    AppHandle, Runtime,
};

use crate::models::*;

use std::{collections::HashMap, sync::Arc};

pub use ffi::NotificationPlugin;

/// Validation checks for macOS notifications functionality.
///
/// UserNotifications requires the app to run from a signed .app bundle.
/// During development with `tauri dev`, the binary runs
/// directly without a bundle, causing UserNotifications calls to fail silently or crash.
mod validation {
    /// Ensures the app is running from a .app bundle.
    pub fn require_bundle() -> crate::Result<()> {
        std::env::current_exe()
            .ok()
            .and_then(|exe| {
                let macos = exe.parent()?;
                let contents = macos.parent()?;
                let bundle = contents.parent()?;
                (macos.ends_with("MacOS")
                    && contents.ends_with("Contents")
                    && bundle.to_string_lossy().ends_with(".app"))
                .then_some(())
            })
            .ok_or_else(|| {
                crate::error::PluginInvokeError::InvokeRejected(crate::error::ErrorResponse {
                    code: None,
                    message: Some("Notifications plugin requires the app to run from a .app bundle. You can enable notify-rust feature for development.".to_string()),
                    data: (),
                })
                .into()
            })
    }
}

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
        async fn registerForPushNotifications(&self) -> Result<String, FFIResult>;
        fn unregisterForPushNotifications(&self) -> Result<(), FFIResult>;
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

impl std::fmt::Debug for ffi::NotificationPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotificationPlugin").finish()
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
    validation::require_bundle()?;

    Ok(Notifications {
        app: app.clone(),
        plugin: Arc::new(ffi::NotificationPlugin::init_plugin()),
    })
}

impl<R: Runtime> crate::NotificationsBuilder<R> {
    pub async fn show(self) -> crate::Result<()> {
        validation::require_bundle()?;

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
        validation::require_bundle()?;

        let response: crate::PermissionResponse = self.plugin.requestPermissions().await.parse()?;
        Ok(response.permission_state)
    }

    pub async fn register_for_push_notifications(&self) -> crate::Result<String> {
        validation::require_bundle()?;

        #[cfg(feature = "push-notifications")]
        {
            let response: crate::PushNotificationResponse =
                self.plugin.registerForPushNotifications().await.parse()?;
            Ok(response.device_token)
        }
        #[cfg(not(feature = "push-notifications"))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Push notifications feature is not enabled",
            )))
        }
    }

    pub fn unregister_for_push_notifications(&self) -> crate::Result<()> {
        validation::require_bundle()?;

        #[cfg(feature = "push-notifications")]
        {
            self.plugin.unregisterForPushNotifications().parse_void()
        }
        #[cfg(not(feature = "push-notifications"))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Push notifications feature is not enabled",
            )))
        }
    }

    pub async fn permission_state(&self) -> crate::Result<PermissionState> {
        validation::require_bundle()?;

        let response: crate::PermissionResponse = self.plugin.checkPermissions().await.parse()?;
        Ok(response.permission_state)
    }

    pub fn register_action_types(&self, types: Vec<ActionType>) -> crate::Result<()> {
        validation::require_bundle()?;

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
        validation::require_bundle()?;

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
        validation::require_bundle()?;

        self.plugin.getActive().await.parse()
    }

    pub fn remove_all_active(&self) -> crate::Result<()> {
        validation::require_bundle()?;

        self.plugin.removeAllActive().parse_void()
    }

    pub async fn pending(&self) -> crate::Result<Vec<PendingNotification>> {
        validation::require_bundle()?;

        self.plugin.getPending().await.parse()
    }

    /// Cancel pending notifications.
    pub fn cancel(&self, notifications: Vec<i32>) -> crate::Result<()> {
        validation::require_bundle()?;

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
        validation::require_bundle()?;

        self.plugin.cancelAll().parse_void()
    }

    /// Set click listener active state.
    /// Used internally to track if JS listener is registered.
    pub fn set_click_listener_active(&self, active: bool) -> crate::Result<()> {
        validation::require_bundle()?;

        let mut args = HashMap::new();
        args.insert("active", active);
        self.plugin
            .setClickListenerActive(
                serde_json::to_string(&args)
                    .map_err(|e| crate::error::PluginInvokeError::CannotSerializePayload(e))?,
            )
            .parse_void()
    }

    /// Create a notification channel (not supported on macOS).
    pub fn create_channel(&self, _channel: crate::Channel) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported on macOS",
        )))
    }

    /// Delete a notification channel (not supported on macOS).
    pub fn delete_channel(&self, _id: impl Into<String>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported on macOS",
        )))
    }

    /// List notification channels (not supported on macOS).
    pub fn list_channels(&self) -> crate::Result<Vec<crate::Channel>> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported on macOS",
        )))
    }
}
