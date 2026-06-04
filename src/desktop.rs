use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PermissionState, PluginApi},
    AppHandle, Runtime,
};

use crate::NotificationsBuilder;

// Signature must match the iOS/Android `init` so the cfg-gated call sites in `lib.rs::init` compile uniformly.
#[allow(clippy::unnecessary_wraps)]
pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Notifications<R>> {
    Ok(Notifications {
        app: app.clone(),
        #[cfg(all(target_os = "linux", feature = "push-notifications"))]
        unifiedpush: tokio::sync::OnceCell::new(),
    })
}

/// Access to the notification APIs.
///
/// You can get an instance of this type via [`NotificationsExt`](crate::NotificationsExt)
pub struct Notifications<R: Runtime> {
    app: AppHandle<R>,
    #[cfg(all(target_os = "linux", feature = "push-notifications"))]
    unifiedpush: tokio::sync::OnceCell<std::sync::Arc<crate::unifiedpush::UnifiedPushState>>,
}

#[cfg(all(target_os = "linux", feature = "push-notifications"))]
impl<R: Runtime> Notifications<R> {
    async fn unifiedpush_state(
        &self,
    ) -> crate::Result<&std::sync::Arc<crate::unifiedpush::UnifiedPushState>> {
        self.unifiedpush
            .get_or_try_init(|| crate::unifiedpush::UnifiedPushState::new(&self.app))
            .await
    }
}

// `async` and `Result` mirror the mobile/macOS plugin API so callers can `.await` and `?` uniformly.
#[allow(clippy::unused_async, clippy::unnecessary_wraps)]
impl<R: Runtime> crate::NotificationsBuilder<R> {
    pub async fn show(self) -> crate::Result<()> {
        let mut notification = imp::Notification::new(self.app.config().identifier.clone());

        if let Some(title) = self
            .data
            .title
            .or_else(|| self.app.config().product_name.clone())
        {
            notification = notification.title(title);
        }
        if let Some(body) = self.data.body {
            notification = notification.body(body);
        }
        if let Some(icon) = self.data.icon {
            notification = notification.icon(icon);
        }

        notification.show()?;

        Ok(())
    }
}

// `async` mirrors the mobile/macOS plugin API so callers can `.await` uniformly.
#[allow(clippy::unused_async)]
impl<R: Runtime> Notifications<R> {
    pub fn builder(&self) -> NotificationsBuilder<R> {
        NotificationsBuilder::new(self.app.clone())
    }

    pub async fn request_permission(&self) -> crate::Result<PermissionState> {
        Ok(PermissionState::Granted)
    }

    /// On Linux with the `push-notifications` feature this registers with the
    /// selected (or first available) `UnifiedPush` distributor and returns the
    /// endpoint URL. Apps that need endpoint stability across launches should
    /// call [`set_token`](Self::set_token) before this with a persisted token.
    pub async fn register_for_push_notifications(&self) -> crate::Result<String> {
        #[cfg(all(target_os = "linux", feature = "push-notifications"))]
        {
            let state = self.unifiedpush_state().await?;
            state.register().await
        }
        #[cfg(not(all(target_os = "linux", feature = "push-notifications")))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Push notifications are not supported on desktop platforms",
            )))
        }
    }

    /// Sync signature preserved for source compatibility — callers that need
    /// the Linux `UnifiedPush` unregister path should use
    /// [`unregister_for_push_notifications_async`] instead.
    pub fn unregister_for_push_notifications(&self) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Push notifications are not supported on desktop platforms",
        )))
    }

    /// Async unregister used by the Tauri command bridge. On Linux with the
    /// `push-notifications` feature this calls
    /// `org.unifiedpush.Distributor1.Unregister` and clears the in-memory
    /// active registration.
    pub async fn unregister_for_push_notifications_async(&self) -> crate::Result<()> {
        #[cfg(all(target_os = "linux", feature = "push-notifications"))]
        {
            if let Some(state) = self.unifiedpush.get() {
                state.unregister().await?;
            }
            Ok(())
        }
        #[cfg(not(all(target_os = "linux", feature = "push-notifications")))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Push notifications are not supported on desktop platforms",
            )))
        }
    }

    /// Lists currently running `UnifiedPush` distributors. Linux-only.
    #[cfg(all(target_os = "linux", feature = "push-notifications"))]
    pub async fn list_distributors(&self) -> crate::Result<Vec<String>> {
        let state = self.unifiedpush_state().await?;
        state.list_distributors().await
    }

    /// Pins the chosen `UnifiedPush` distributor for this process. Linux-only.
    #[cfg(all(target_os = "linux", feature = "push-notifications"))]
    pub async fn set_distributor(&self, name: String) -> crate::Result<()> {
        let state = self.unifiedpush_state().await?;
        state.set_distributor(name).await
    }

    /// Sets the `UnifiedPush` client token used on subsequent register calls.
    /// Pass the same token across launches to keep the endpoint URL stable.
    /// Linux-only.
    #[cfg(all(target_os = "linux", feature = "push-notifications"))]
    pub async fn set_token(&self, token: String) -> crate::Result<()> {
        let state = self.unifiedpush_state().await?;
        state.set_token(token).await
    }

    pub async fn permission_state(&self) -> crate::Result<PermissionState> {
        Ok(PermissionState::Granted)
    }

    pub async fn pending(&self) -> crate::Result<Vec<crate::PendingNotification>> {
        Err(crate::Error::Io(std::io::Error::other(
            "Pending notifications are not supported with notify-rust",
        )))
    }

    pub async fn active(&self) -> crate::Result<Vec<crate::ActiveNotification>> {
        Err(crate::Error::Io(std::io::Error::other(
            "Active notifications are not supported with notify-rust",
        )))
    }

    pub fn set_click_listener_active(&self, _active: bool) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Click listeners are not supported with notify-rust",
        )))
    }

    pub fn remove_active(&self, _ids: Vec<i32>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Removing active notifications is not supported with notify-rust",
        )))
    }

    pub fn cancel(&self, _notifications: Vec<i32>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Canceling notifications is not supported with notify-rust",
        )))
    }

    pub fn cancel_all(&self) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Canceling notifications is not supported with notify-rust",
        )))
    }

    pub fn register_action_types(&self, _types: Vec<crate::ActionType>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Action types are not supported with notify-rust",
        )))
    }

    pub fn create_channel(&self, _channel: crate::Channel) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported with notify-rust",
        )))
    }

    pub fn delete_channel(&self, _id: impl Into<String>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported with notify-rust",
        )))
    }

    pub fn list_channels(&self) -> crate::Result<Vec<crate::Channel>> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported with notify-rust",
        )))
    }
}

mod imp {
    //! Types and functions related to desktop notifications.

    #[cfg(windows)]
    use std::path::MAIN_SEPARATOR as SEP;

    /// The desktop notification definition.
    ///
    /// Allows you to construct a Notification data and send it.
    #[allow(dead_code)]
    #[derive(Debug, Default)]
    pub struct Notification {
        /// The notification body.
        body: Option<String>,
        /// The notification title.
        title: Option<String>,
        /// The notification icon.
        icon: Option<String>,
        /// The notification identifier
        identifier: String,
    }

    impl Notification {
        /// Initializes a instance of a Notification.
        pub fn new(identifier: impl Into<String>) -> Self {
            Self {
                identifier: identifier.into(),
                ..Default::default()
            }
        }

        /// Sets the notification body.
        #[must_use]
        pub fn body(mut self, body: impl Into<String>) -> Self {
            self.body = Some(body.into());
            self
        }

        /// Sets the notification title.
        #[must_use]
        pub fn title(mut self, title: impl Into<String>) -> Self {
            self.title = Some(title.into());
            self
        }

        /// Sets the notification icon.
        #[must_use]
        pub fn icon(mut self, icon: impl Into<String>) -> Self {
            self.icon = Some(icon.into());
            self
        }

        /// Shows the notification.
        // `current_exe()?` returns Result on Windows; Result is kept for that branch.
        #[allow(clippy::unnecessary_wraps)]
        pub fn show(self) -> crate::Result<()> {
            let mut notification = notify_rust::Notification::new();
            if let Some(body) = self.body {
                notification.body(&body);
            }
            if let Some(title) = self.title {
                notification.summary(&title);
            }
            if let Some(icon) = self.icon {
                notification.icon(&icon);
            } else {
                notification.auto_icon();
            }
            #[cfg(windows)]
            {
                let exe = tauri::utils::platform::current_exe()?;
                let exe_dir = exe.parent().expect("failed to get exe directory");
                let curr_dir = exe_dir.display().to_string();
                // set the notification's System.AppUserModel.ID only when running the installed app
                if !(curr_dir.ends_with(format!("{SEP}target{SEP}debug").as_str())
                    || curr_dir.ends_with(format!("{SEP}target{SEP}release").as_str()))
                {
                    notification.app_id(&self.identifier);
                }
            }
            #[cfg(target_os = "macos")]
            {
                let _ = notify_rust::set_application(if tauri::is_dev() {
                    "com.apple.Terminal"
                } else {
                    &self.identifier
                });
            }

            // `notify_rust::Notification::show()` is sync and runs an internal
            // blocking D-Bus call (via zbus's `block_on`). Calling it inside
            // `async_runtime::spawn` panics with "Cannot start a runtime from
            // within a runtime" — `spawn_blocking` parks the call on a
            // dedicated blocking thread so the nested runtime is fine.
            //
            // We deliberately leak the returned `NotificationHandle` (and the
            // D-Bus `Connection` it owns). Some Linux notification daemons
            // (mako, swaync, some dunst configs) treat the sending client
            // disconnecting as a cue to dismiss its open popups, which would
            // make our notifications flash for milliseconds before vanishing.
            tauri::async_runtime::spawn_blocking(move || match notification.show() {
                Ok(handle) => {
                    std::mem::forget(handle);
                }
                Err(e) => log::warn!("Failed to show notification: {e}"),
            });

            Ok(())
        }
    }
}
