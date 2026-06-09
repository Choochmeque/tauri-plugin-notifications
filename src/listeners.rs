//! Shared listener management for desktop platforms.
//!
//! This is a replication of Tauri's plugin listener implementation which is
//! currently only available for mobile plugins. Once Tauri adds desktop support
//! for plugin listeners, this module can be removed.
//!
//! Provides channel-based event delivery for notification events such as
//! notification received, action performed, and notification clicked.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

#[cfg(all(target_os = "windows", not(feature = "notify-rust")))]
use tauri::Manager;
use tauri::{AppHandle, Runtime};

use crate::error::{ErrorResponse, PluginInvokeError};

type ChannelMap = HashMap<u32, tauri::ipc::Channel<serde_json::Value>>;
type ListenerMap = HashMap<String, ChannelMap>;

static LISTENERS: OnceLock<RwLock<ListenerMap>> = OnceLock::new();

/// Initialize the listeners registry. Call this during plugin init.
pub fn init() {
    let _ = LISTENERS.get_or_init(|| RwLock::new(HashMap::new()));
}

/// Trigger an event to all registered listeners for the given event name.
///
/// Called by platform-specific code when notification events occur.
// Owned `payload` is taken from the FFI bridge in `macos.rs`.
#[allow(dead_code, clippy::needless_pass_by_value)]
pub fn trigger(event: &str, payload: String) -> crate::Result<()> {
    let listeners = LISTENERS.get().ok_or_else(|| {
        crate::Error::from(PluginInvokeError::InvokeRejected(ErrorResponse {
            code: None,
            message: Some("Listeners not initialized".to_string()),
            data: (),
        }))
    })?;

    let channels: Vec<tauri::ipc::Channel<serde_json::Value>> = {
        let guard = listeners.read().map_err(|e| {
            crate::Error::from(PluginInvokeError::InvokeRejected(ErrorResponse {
                code: None,
                message: Some(format!("Failed to acquire read lock: {e}")),
                data: (),
            }))
        })?;
        guard
            .get(event)
            .map(|c| c.values().cloned().collect())
            .unwrap_or_default()
    };

    if !channels.is_empty() {
        let value: serde_json::Value = serde_json::from_str(&payload).map_err(|e| {
            crate::Error::from(PluginInvokeError::InvokeRejected(ErrorResponse {
                code: None,
                message: Some(format!("Failed to parse payload JSON: {e}")),
                data: (),
            }))
        })?;
        for channel in &channels {
            let _ = channel.send(value.clone());
        }
    }
    Ok(())
}

/// Register a channel to receive events for the given event name.
///
/// On Windows, subscribing to `notificationClicked` synchronously drains any
/// cold-start activation payload buffered by the COM activator before the JS
/// listener was up. That makes the `push-listener.tsx` contract ("subscribing
/// flushes the buffered tap") work without the app calling any extra command.
// Tauri commands receive serde-deserialized owned values.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn register_listener<R: Runtime>(
    app: AppHandle<R>,
    event: String,
    handler: tauri::ipc::Channel<serde_json::Value>,
) -> crate::Result<()> {
    let listeners = LISTENERS.get_or_init(|| RwLock::new(HashMap::new()));
    let should_drain_clicks = event == "notificationClicked";
    {
        let mut guard = listeners.write().map_err(|e| {
            crate::Error::from(PluginInvokeError::InvokeRejected(ErrorResponse {
                code: None,
                message: Some(format!("Failed to acquire write lock: {e}")),
                data: (),
            }))
        })?;
        guard
            .entry(event)
            .or_default()
            .insert(handler.id(), handler);
    }
    #[cfg(all(target_os = "windows", not(feature = "notify-rust")))]
    if should_drain_clicks {
        if let Some(notif) = app.try_state::<crate::Notifications<R>>() {
            notif.drain_pending_clicks();
        }
    }
    #[cfg(not(all(target_os = "windows", not(feature = "notify-rust"))))]
    let _ = (app, should_drain_clicks);
    Ok(())
}

/// Remove a previously registered listener by event name and channel ID.
// Tauri commands receive serde-deserialized owned values.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn remove_listener(event: String, channel_id: u32) -> crate::Result<()> {
    let listeners = LISTENERS.get().ok_or_else(|| {
        crate::Error::from(PluginInvokeError::InvokeRejected(ErrorResponse {
            code: None,
            message: Some("Listeners not initialized".to_string()),
            data: (),
        }))
    })?;
    {
        let mut guard = listeners.write().map_err(|e| {
            crate::Error::from(PluginInvokeError::InvokeRejected(ErrorResponse {
                code: None,
                message: Some(format!("Failed to acquire write lock: {e}")),
                data: (),
            }))
        })?;
        if let Some(channels) = guard.get_mut(&event) {
            channels.remove(&channel_id);
        }
    }
    Ok(())
}
