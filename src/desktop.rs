use serde::de::DeserializeOwned;
use tauri::{
    AppHandle, Runtime,
    plugin::{PermissionState, PluginApi},
};

use crate::NotificationsBuilder;

/// Tracks a single live `notify-rust` notification on Linux. Owning the
/// `NotificationHandle` keeps the underlying D-Bus `Connection` alive
/// (preventing the "popup disappears when the sending client disconnects"
/// behavior some Linux daemons exhibit) and lets us implement
/// `active`/`cancel` for the caller-supplied id.
///
/// macOS / Windows: `notify_rust::NotificationHandle` on those platforms
/// doesn't expose a useful `close()` (macOS daemon doesn't dismiss on
/// sender disconnect; Windows's handle is a thin wrapper without close
/// semantics), so we don't track there and the active-list / cancel
/// methods stay as the existing stubs.
#[cfg(target_os = "linux")]
struct ActiveEntry {
    caller_id: i32,
    /// Shared with the action waiter spawned by [`NotificationsBuilder::show`]
    /// when a click listener is subscribed, so both can hold the handle for as
    /// long as the notification is live.
    handle: std::sync::Arc<notify_rust::NotificationHandle>,
    title: Option<String>,
    body: Option<String>,
}

// Signature must match the iOS/Android `init` so the cfg-gated call sites in `lib.rs::init` compile uniformly.
#[allow(clippy::unnecessary_wraps)]
pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Notifications<R>> {
    Ok(Notifications {
        app: app.clone(),
        #[cfg(target_os = "linux")]
        active: std::sync::Mutex::new(std::collections::HashMap::new()),
        #[cfg(target_os = "linux")]
        active_counter: std::sync::atomic::AtomicU64::new(0),
        #[cfg(all(target_os = "linux", feature = "push-notifications"))]
        unifiedpush: tokio::sync::OnceCell::new(),
        click_listener_active: std::sync::atomic::AtomicBool::new(false),
    })
}

/// Access to the notification APIs.
///
/// You can get an instance of this type via [`NotificationsExt`](crate::NotificationsExt)
pub struct Notifications<R: Runtime> {
    app: AppHandle<R>,
    /// Currently-displayed notifications, keyed by an internal monotonic
    /// counter (not the caller-supplied id, so multiple notifications with
    /// the same id coexist without evicting each other). Holding the handles
    /// keeps the popups visible and lets `cancel`/`cancel_all`/
    /// `remove_active`/`active` work without leaking. Entries are removed by
    /// explicit cancel; expired/auto-dismissed notifications may linger
    /// because notify-rust doesn't expose a non-consuming "closed" callback.
    #[cfg(target_os = "linux")]
    active: std::sync::Mutex<std::collections::HashMap<u64, ActiveEntry>>,
    #[cfg(target_os = "linux")]
    active_counter: std::sync::atomic::AtomicU64,
    #[cfg(all(target_os = "linux", feature = "push-notifications"))]
    unifiedpush: tokio::sync::OnceCell<std::sync::Arc<crate::unifiedpush::UnifiedPushState>>,
    /// Whether a JS `notificationClicked` listener is currently subscribed.
    /// Set by [`Notifications::set_click_listener_active`], which the JS
    /// `onNotificationClicked` helper calls right after registering. Read by
    /// [`NotificationsBuilder::show`] to decide whether to observe the
    /// notification's response, and — on macOS and Linux — whether the
    /// notification needs an activation action declared at all.
    click_listener_active: std::sync::atomic::AtomicBool,
}

#[cfg(target_os = "linux")]
fn active_lock_err(e: impl std::fmt::Display) -> crate::Error {
    crate::Error::Io(std::io::Error::other(format!(
        "active notifications mutex poisoned: {e}"
    )))
}

#[cfg(target_os = "linux")]
impl<R: Runtime> Notifications<R> {
    /// Finds every tracked notification whose caller id is in `caller_ids`,
    /// removes them from the active map, and spawns `handle.close_async()` so
    /// the command call returns quickly.
    fn close_by_caller_ids(&self, caller_ids: &[i32]) -> crate::Result<()> {
        let mut to_close: Vec<ActiveEntry> = Vec::new();
        {
            let mut active = self.active.lock().map_err(active_lock_err)?;
            // Move the map out, partition entries into "close" vs "keep" in
            // one pass, then put the kept ones back. Avoids the borrow-
            // checker dance of iter-then-remove (which would need a
            // throwaway `Vec<u64>` of keys) without holding the lock any
            // longer than necessary.
            let kept: std::collections::HashMap<u64, ActiveEntry> = std::mem::take(&mut *active)
                .into_iter()
                .filter_map(|(k, entry)| {
                    if caller_ids.contains(&entry.caller_id) {
                        to_close.push(entry);
                        None
                    } else {
                        Some((k, entry))
                    }
                })
                .collect();
            *active = kept;
        }
        for entry in to_close {
            // `close_async` borrows, so it works through the `Arc` that the
            // action waiter may also be holding; the consuming `close()` can't.
            tauri::async_runtime::spawn(async move { entry.handle.close_async().await });
        }
        Ok(())
    }
}

#[cfg(all(target_os = "linux", feature = "push-notifications"))]
impl<R: Runtime> Notifications<R> {
    async fn unifiedpush_state(
        &self,
    ) -> crate::Result<&std::sync::Arc<crate::unifiedpush::UnifiedPushState>> {
        self.unifiedpush
            .get_or_try_init(|| {
                let displayer = Self::build_push_displayer(self.app.clone());
                crate::unifiedpush::UnifiedPushState::new(&self.app, Some(displayer))
            })
            .await
    }

    /// Builds the `PushDisplayer` callback handed to `UnifiedPushState`. The
    /// callback runs `notify_rust::Notification::show()` on a blocking thread
    /// and routes the resulting handle into the same `active` map that local
    /// notifications use, so push toasts:
    ///   * Stay visible (handle is held → D-Bus connection stays alive →
    ///     daemons don't dismiss-on-disconnect).
    ///   * Show up in [`Notifications::active`] alongside local notifications.
    ///   * Can be cancelled via the existing `cancel`/`cancel_all` methods
    ///     (caller id is `0` because `UnifiedPush` messages don't carry one).
    fn build_push_displayer(app: AppHandle<R>) -> crate::unifiedpush::PushDisplayer {
        std::sync::Arc::new(move |title: Option<String>, body: Option<String>| {
            let app = app.clone();
            let identifier = app.config().identifier.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let notification = match imp::build_notification(
                    title.as_deref(),
                    body.as_deref(),
                    None,
                    &identifier,
                ) {
                    Ok(n) => n,
                    Err(e) => {
                        log::warn!("Failed to build push notification: {e}");
                        return;
                    }
                };
                match notification.show() {
                    Ok(handle) => {
                        use std::sync::atomic::Ordering;
                        use tauri::Manager;
                        let state = app.state::<Self>();
                        let entry_id = state.active_counter.fetch_add(1, Ordering::Relaxed);
                        let entry = ActiveEntry {
                            caller_id: 0,
                            // No click waiter for push toasts (they carry no
                            // caller id or extras), but the field is shared with
                            // the local path, which does spawn one.
                            handle: std::sync::Arc::new(handle),
                            title,
                            body,
                        };
                        let lock = state.active.lock();
                        match lock {
                            Ok(mut active) => {
                                active.insert(entry_id, entry);
                            }
                            Err(poisoned) => {
                                log::warn!("active notifications mutex was poisoned; recovering");
                                poisoned.into_inner().insert(entry_id, entry);
                            }
                        }
                    }
                    Err(e) => log::warn!("Failed to show push notification toast: {e}"),
                }
            });
        })
    }
}

// `async` and `Result` mirror the mobile/macOS plugin API so callers can `.await` and `?` uniformly.
impl<R: Runtime> crate::NotificationsBuilder<R> {
    pub async fn show(self) -> crate::Result<()> {
        let caller_id = self.data.id;
        let title = self
            .data
            .title
            .or_else(|| self.app.config().product_name.clone());
        let body = self.data.body;
        let icon = self.data.icon;
        // Captured for the click payload: the response from `notify-rust`
        // carries no notification identity, so the waiter has to close over the
        // id and extras of the notification it was spawned for.
        let extra = self.data.extra;
        let identifier = self.app.config().identifier.clone();
        let app = self.app.clone();

        // Read before building: on macOS and Linux, whether a listener is
        // subscribed changes how the notification itself has to be constructed.
        let click_listener_active = {
            use tauri::Manager;
            app.state::<Notifications<R>>()
                .click_listener_active
                .load(std::sync::atomic::Ordering::Relaxed)
        };

        #[cfg_attr(
            target_os = "windows",
            expect(unused_mut, reason = "no action is declared on Windows")
        )]
        let mut notification = imp::build_notification(
            title.as_deref(),
            body.as_deref(),
            icon.as_deref(),
            &identifier,
        )?;

        // Both non-Windows backends need an action declared before a click can
        // ever be reported, for different reasons:
        //
        // * macOS — `mac-notification-sys` only blocks for a response when the
        //   notification declares an interactive element (`needs_response()` is
        //   `main_button.is_some() || close_button.is_some() || wait_for_click`).
        //   notify-rust never sets `wait_for_click` and only sets `close_button`
        //   for `on_close`, so without an action `wait_for_response` returns
        //   `None` immediately. One action becomes `MainButton::SingleAction`,
        //   which arms the wait.
        // * Linux — per the freedesktop spec the client declares the activation
        //   action, and the daemon reports a body click as the action keyed
        //   `"default"`. Without it there's no `ActionInvoked` signal to catch.
        //   Requires the daemon to advertise the `actions` capability.
        //
        // Windows needs none of this: notify-rust wires `on_activated` /
        // `on_dismissed` into an mpsc channel unconditionally, and a body tap
        // arrives with empty `Arguments` — declaring an action would only add a
        // visible button.
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        if click_listener_active {
            notification.action("default", "Open");
        }

        // `notify_rust::Notification::show()` is sync and runs an internal
        // blocking D-Bus call (via zbus's `block_on`). Calling it inside
        // `async_runtime::spawn` panics with "Cannot start a runtime from
        // within a runtime"; `spawn_blocking` parks it on a blocking thread.
        // We `.await` the join so we can capture the handle for tracking and
        // surface any error to the caller.
        let join_result = tauri::async_runtime::spawn_blocking(move || notification.show())
            .await
            .map_err(|e| {
                crate::Error::Io(std::io::Error::other(format!(
                    "notification spawn_blocking join error: {e}"
                )))
            })?;

        match join_result {
            #[cfg(target_os = "linux")]
            Ok(handle) => {
                use std::sync::atomic::Ordering;
                use tauri::Manager;
                // `Arc` because the handle has two concurrent users: the active
                // map (for `cancel`) and the action waiter. Both only ever need
                // `&self` — `wait_for_action_async` and `close_async` borrow,
                // unlike their consuming sync counterparts.
                let handle = std::sync::Arc::new(handle);
                if click_listener_active {
                    let waiter = std::sync::Arc::clone(&handle);
                    tauri::async_runtime::spawn(async move {
                        waiter
                            .wait_for_action_async(|response| {
                                imp::emit_click(caller_id, &extra, response);
                            })
                            .await;
                    });
                }
                let state = app.state::<Notifications<R>>();
                let entry_id = state.active_counter.fetch_add(1, Ordering::Relaxed);
                let entry = ActiveEntry {
                    caller_id,
                    handle,
                    title,
                    body,
                };
                // Take the lock into a binding so its `MutexGuard` temporary
                // doesn't outlive `state` in the `match` arms.
                let lock_result = state.active.lock();
                match lock_result {
                    Ok(mut active) => {
                        active.insert(entry_id, entry);
                    }
                    Err(poisoned) => {
                        log::warn!("active notifications mutex was poisoned; recovering");
                        poisoned.into_inner().insert(entry_id, entry);
                    }
                }
            }
            // macOS / Windows: both wait on a blocking thread, for different
            // reasons.
            //
            // * macOS — `show()` posts nothing here, it only wraps the
            //   `Notification`. Delivery happens in the handle's `Drop`
            //   (fire-and-forget) or inside `wait_for_response`, which posts and
            //   then blocks. So the listener decides which path delivers it, and
            //   the `else` branch below is what shows the notification.
            //   `mac-notification-sys` blocks on a condvar when called off the
            //   main thread (it only spins the run loop when it *is* the main
            //   thread), so the parked thread costs no CPU.
            // * Windows — the toast is already posted; the handle owns the
            //   receiving end of the mpsc channel notify-rust's `on_activated` /
            //   `on_dismissed` feed. WinRT delivers activations on a threadpool
            //   thread, so this is a plain blocking `recv()` that doesn't need
            //   the showing thread to pump messages. Dropping the handle is
            //   harmless: the sends just fail silently.
            //
            // `wait_for_action` is deliberately not used: its `Ok(_)` arm
            // collapses `Default` into `"__closed"`, which would make a body tap
            // indistinguishable from a dismissal.
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            Ok(handle) => {
                let _ = (title, body, app);
                if click_listener_active {
                    tauri::async_runtime::spawn_blocking(move || {
                        let responded = handle.wait_for_response(
                            |response: &notify_rust::NotificationResponse| {
                                imp::emit_click(caller_id, &extra, response);
                            },
                        );
                        if let Err(e) = responded {
                            log::warn!("Failed to await notification response: {e}");
                        }
                    });
                } else {
                    drop(handle);
                }
            }
            // Propagate the underlying `notify-rust` failure (missing
            // notification daemon, D-Bus permission denied, etc.) instead of
            // swallowing it — matches the mobile/macOS behavior and lets JS
            // callers handle delivery failures.
            Err(e) => {
                return Err(crate::Error::Io(std::io::Error::other(format!(
                    "Failed to show notification: {e}"
                ))));
            }
        }

        Ok(())
    }
}

// `async` mirrors the mobile/macOS plugin API so callers can `.await` uniformly.
#[allow(unknown_lints, clippy::unused_async, clippy::unused_async_trait_impl)]
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

    /// Linux: returns the currently-tracked notifications. The list is
    /// populated by [`NotificationsBuilder::show`] and pruned by
    /// `cancel`/`cancel_all`/`remove_active`. Entries dismissed by the user
    /// or expired by the OS may linger until the next explicit cancel call,
    /// since notify-rust doesn't expose a non-consuming "closed" callback.
    ///
    /// macOS / Windows: still unsupported.
    pub async fn active(&self) -> crate::Result<Vec<crate::ActiveNotification>> {
        #[cfg(target_os = "linux")]
        {
            let active = self.active.lock().map_err(active_lock_err)?;
            Ok(active
                .values()
                .map(|entry| {
                    crate::ActiveNotification::new(
                        entry.caller_id,
                        entry.title.clone(),
                        entry.body.clone(),
                    )
                })
                .collect())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Active notifications are not supported with notify-rust",
            )))
        }
    }

    /// Records whether a `notificationClicked` listener is subscribed.
    ///
    /// Notifications shown while this is `true` are observed for the user's
    /// response and emit `notificationClicked` on activation (see
    /// [`NotificationsBuilder::show`]). Notifications already on screen when it
    /// flips are not retroactively observed, and there is no cold-start buffer —
    /// unlike the native Windows backend, this one can't be launched by a click.
    pub fn set_click_listener_active(&self, active: bool) -> crate::Result<()> {
        self.click_listener_active
            .store(active, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /// Linux: closes every tracked notification whose caller-supplied id
    /// appears in `ids` and removes it from the active map.
    /// macOS / Windows: unsupported.
    // Existing public signature; switching to `&[i32]` would be breaking.
    #[allow(clippy::needless_pass_by_value)]
    pub fn remove_active(&self, ids: Vec<i32>) -> crate::Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.close_by_caller_ids(&ids)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = ids;
            Err(crate::Error::Io(std::io::Error::other(
                "Removing active notifications is not supported with notify-rust",
            )))
        }
    }

    pub fn remove_all_active(&self) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Removing active notifications is not supported with notify-rust",
        )))
    }

    /// Same semantics as [`remove_active`](Self::remove_active) on Linux;
    /// macOS / Windows: unsupported.
    // Existing public signature; switching to `&[i32]` would be breaking.
    #[allow(clippy::needless_pass_by_value)]
    pub fn cancel(&self, notifications: Vec<i32>) -> crate::Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.close_by_caller_ids(&notifications)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = notifications;
            Err(crate::Error::Io(std::io::Error::other(
                "Canceling notifications is not supported with notify-rust",
            )))
        }
    }

    /// Linux: closes every tracked notification.
    /// macOS / Windows: unsupported.
    pub fn cancel_all(&self) -> crate::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let drained: Vec<ActiveEntry> = {
                let mut active = self.active.lock().map_err(active_lock_err)?;
                active.drain().map(|(_, v)| v).collect()
            };
            for entry in drained {
                tauri::async_runtime::spawn(async move { entry.handle.close_async().await });
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Canceling notifications is not supported with notify-rust",
            )))
        }
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
    //! Helpers for assembling the cross-platform `notify_rust::Notification`
    //! before handing it off to a blocking thread for delivery.

    /// Dispatches a `notificationClicked` event for a body tap, matching the
    /// payload the native backends emit (`{ id, data }`, see `windows.rs`).
    ///
    /// A body tap arrives as
    /// [`NotificationResponse::Default`](notify_rust::NotificationResponse::Default).
    /// On macOS and Linux, arming the wait requires declaring a `"default"`
    /// action (see `NotificationsBuilder::show`), so tapping that button instead
    /// arrives as `Action("default")` — both mean "the user activated this
    /// notification". `Closed` is ignored, so dismissal and expiry fire nothing.
    pub fn emit_click(
        id: i32,
        extra: &std::collections::HashMap<String, serde_json::Value>,
        response: &notify_rust::NotificationResponse,
    ) {
        use notify_rust::NotificationResponse;

        let activated = match response {
            NotificationResponse::Default => true,
            NotificationResponse::Action(key) => key == "default",
            _ => false,
        };
        if !activated {
            log::debug!("notification {id} closed without activation: {response:?}");
            return;
        }
        let payload = serde_json::json!({ "id": id, "data": extra });
        if let Err(e) = crate::listeners::trigger("notificationClicked", payload.to_string()) {
            log::warn!("Failed to dispatch notificationClicked: {e}");
        }
    }

    #[cfg(windows)]
    use std::path::MAIN_SEPARATOR as SEP;

    /// Builds a fully-configured `notify_rust::Notification` from the parts
    /// the cross-platform builder produced. Returns an error only on Windows
    /// if `current_exe` lookup fails; other platforms are infallible — the
    /// `Result` wrapper exists for the Windows branch only.
    #[allow(clippy::unnecessary_wraps)]
    pub fn build_notification(
        title: Option<&str>,
        body: Option<&str>,
        icon: Option<&str>,
        identifier: &str,
    ) -> crate::Result<notify_rust::Notification> {
        let mut notification = notify_rust::Notification::new();
        if let Some(body) = body {
            notification.body(body);
        }
        if let Some(title) = title {
            notification.summary(title);
        }
        if let Some(icon) = icon {
            notification.icon(icon);
        } else {
            notification.auto_icon();
        }

        #[cfg(windows)]
        {
            let exe = tauri::utils::platform::current_exe()?;
            let exe_dir = exe.parent().expect("failed to get exe directory");
            let curr_dir = exe_dir.display().to_string();
            // Only set System.AppUserModel.ID on the installed app, not when
            // running from `cargo`'s target dirs.
            if !(curr_dir.ends_with(format!("{SEP}target{SEP}debug").as_str())
                || curr_dir.ends_with(format!("{SEP}target{SEP}release").as_str()))
            {
                notification.app_id(identifier);
            }
        }
        #[cfg(target_os = "macos")]
        {
            let _ = notify_rust::set_application(if tauri::is_dev() {
                "com.apple.Terminal"
            } else {
                identifier
            });
        }
        // `identifier` is used by the cfg-gated Windows/macOS branches above
        // — silence the unused-parameter warning on Linux.
        #[cfg(target_os = "linux")]
        let _ = identifier;

        Ok(notification)
    }
}
