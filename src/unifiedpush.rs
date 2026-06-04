//! `UnifiedPush` D-Bus integration for Linux desktop.
//!
//! Implements the connector side of the `UnifiedPush` spec
//! (<https://unifiedpush.org/spec/dbus/>). The plugin owns a `Connection` to the
//! session bus, exposes an `org.unifiedpush.Connector1` interface, and drives
//! `org.unifiedpush.Distributor1` calls against the user-selected distributor.
//!
//! All state is in-memory. The plugin never writes to disk — endpoint stability
//! across launches is the host app's responsibility (pass the same `client_token`
//! to `register_for_push_notifications`).

use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::Duration;

use serde_json::{json, Value as JsonValue};
use tauri::{AppHandle, Runtime};
use tokio::sync::{oneshot, Mutex, RwLock};

const DISTRIBUTOR_PREFIX: &str = "org.unifiedpush.Distributor.";
const CONNECTOR_PATH: &str = "/org/unifiedpush/Connector";
const REGISTER_TIMEOUT_SECS: u64 = 10;

pub const ERR_NO_DISTRIBUTOR: &str = "No UnifiedPush distributor installed — install one from https://unifiedpush.org/users/distributors/";
pub const ERR_REGISTER_TIMEOUT: &str = "Registration timed out";

fn err_distributor_unavailable(name: &str) -> String {
    format!("Distributor '{name}' is not available")
}

fn io_err(msg: impl Into<String>) -> crate::Error {
    crate::Error::Io(std::io::Error::other(msg.into()))
}

#[zbus::proxy(
    interface = "org.unifiedpush.Distributor1",
    default_path = "/org/unifiedpush/Distributor"
)]
trait Distributor {
    fn register(&self, connector: &str, token: &str, vapid: &str)
        -> zbus::Result<(String, String)>;

    fn unregister(&self, token: &str) -> zbus::Result<()>;
}

#[derive(Clone)]
struct ActiveRegistration {
    client_token: String,
    distributor: String,
}

pub struct UnifiedPushState {
    connection: zbus::Connection,
    connector_bus_name: String,
    selected: RwLock<Option<String>>,
    token: RwLock<Option<String>>,
    active: RwLock<Option<ActiveRegistration>>,
    pending: Mutex<HashMap<String, oneshot::Sender<Result<String, String>>>>,
}

impl UnifiedPushState {
    pub async fn new<R: Runtime>(app: &AppHandle<R>) -> crate::Result<Arc<Self>> {
        let connector_bus_name = app.config().identifier.clone();
        if connector_bus_name.is_empty() {
            return Err(io_err(
                "App identifier is empty; cannot register a D-Bus connector name",
            ));
        }

        let connection = zbus::Connection::session()
            .await
            .map_err(|e| io_err(format!("Failed to connect to D-Bus session: {e}")))?;

        let state = Arc::new(Self {
            connection: connection.clone(),
            connector_bus_name: connector_bus_name.clone(),
            selected: RwLock::new(None),
            token: RwLock::new(None),
            active: RwLock::new(None),
            pending: Mutex::new(HashMap::new()),
        });

        let connector = ConnectorService {
            state: Arc::downgrade(&state),
        };
        connection
            .object_server()
            .at(CONNECTOR_PATH, connector)
            .await
            .map_err(|e| io_err(format!("Failed to register connector object: {e}")))?;

        let reply = connection
            .request_name_with_flags(
                connector_bus_name.as_str(),
                zbus::fdo::RequestNameFlags::DoNotQueue
                    | zbus::fdo::RequestNameFlags::ReplaceExisting
                    | zbus::fdo::RequestNameFlags::AllowReplacement,
            )
            .await
            .map_err(|e| {
                io_err(format!(
                    "Failed to request connector bus name '{connector_bus_name}': {e}"
                ))
            })?;

        match reply {
            zbus::fdo::RequestNameReply::PrimaryOwner
            | zbus::fdo::RequestNameReply::AlreadyOwner => {
                log::info!("UnifiedPush connector listening on D-Bus name '{connector_bus_name}'");
            }
            zbus::fdo::RequestNameReply::InQueue => {
                return Err(io_err(format!(
                    "Bus name '{connector_bus_name}' is held by another process; queued instead of becoming primary owner"
                )));
            }
            zbus::fdo::RequestNameReply::Exists => {
                return Err(io_err(format!(
                    "Bus name '{connector_bus_name}' is already held by another process and cannot be replaced"
                )));
            }
        }

        Ok(state)
    }

    pub async fn list_distributors(&self) -> crate::Result<Vec<String>> {
        let dbus = zbus::fdo::DBusProxy::new(&self.connection)
            .await
            .map_err(|e| io_err(format!("Failed to access D-Bus daemon: {e}")))?;
        let names = dbus
            .list_names()
            .await
            .map_err(|e| io_err(format!("Failed to list bus names: {e}")))?;
        let mut out: Vec<String> = names
            .into_iter()
            .map(|n| n.as_str().to_string())
            .filter(|n| n.starts_with(DISTRIBUTOR_PREFIX))
            .collect();
        out.sort();
        out.dedup();
        Ok(out)
    }

    pub async fn set_distributor(&self, name: String) -> crate::Result<()> {
        let distributors = self.list_distributors().await?;
        if !distributors.contains(&name) {
            return Err(io_err(err_distributor_unavailable(&name)));
        }
        *self.selected.write().await = Some(name);
        Ok(())
    }

    /// Sets the client token used on the next `register` call. `UnifiedPush`
    /// distributors derive the endpoint URL from
    /// `(connector_bus_name, client_token)`, so apps that want endpoint
    /// stability across launches should persist this token themselves and
    /// call `set_token` before `register`.
    pub async fn set_token(&self, token: String) -> crate::Result<()> {
        if token.is_empty() {
            return Err(io_err("Token cannot be empty"));
        }
        *self.token.write().await = Some(token);
        Ok(())
    }

    async fn pick_distributor(&self) -> crate::Result<String> {
        let distributors = self.list_distributors().await?;
        let selected = self.selected.read().await.clone();
        if let Some(name) = selected {
            if distributors.contains(&name) {
                return Ok(name);
            }
            return Err(io_err(err_distributor_unavailable(&name)));
        }
        distributors
            .into_iter()
            .next()
            .ok_or_else(|| io_err(ERR_NO_DISTRIBUTOR.to_string()))
    }

    pub async fn register(&self) -> crate::Result<String> {
        let distributor = self.pick_distributor().await?;
        let client_token = self
            .token
            .read()
            .await
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let proxy = DistributorProxy::builder(&self.connection)
            .destination(distributor.clone())
            .map_err(|e| io_err(format!("Invalid distributor name '{distributor}': {e}")))?
            .build()
            .await
            .map_err(|e| {
                io_err(format!(
                    "Failed to connect to distributor '{distributor}': {e}"
                ))
            })?;

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            if pending.contains_key(&client_token) {
                return Err(io_err(
                    "A registration is already in flight for this client token",
                ));
            }
            pending.insert(client_token.clone(), tx);
        }

        let response = proxy
            .register(&self.connector_bus_name, &client_token, "")
            .await;

        match response {
            Ok((code, message)) => {
                if code == "REGISTRATION_FAILED" {
                    self.pending.lock().await.remove(&client_token);
                    let detail = if message.is_empty() {
                        String::new()
                    } else {
                        format!(": {message}")
                    };
                    return Err(io_err(format!("Distributor rejected registration{detail}")));
                }
            }
            Err(e) => {
                self.pending.lock().await.remove(&client_token);
                return Err(io_err(format!("Distributor Register call failed: {e}")));
            }
        }

        let endpoint =
            match tokio::time::timeout(Duration::from_secs(REGISTER_TIMEOUT_SECS), rx).await {
                Ok(Ok(Ok(endpoint))) => endpoint,
                Ok(Ok(Err(reason))) => {
                    return Err(io_err(format!("Registration failed: {reason}")));
                }
                Ok(Err(_)) => {
                    return Err(io_err("Registration callback channel closed".to_string()));
                }
                Err(_) => {
                    self.pending.lock().await.remove(&client_token);
                    return Err(io_err(ERR_REGISTER_TIMEOUT.to_string()));
                }
            };

        *self.active.write().await = Some(ActiveRegistration {
            client_token,
            distributor,
        });
        Ok(endpoint)
    }

    pub async fn unregister(&self) -> crate::Result<()> {
        let Some(active) = self.active.write().await.take() else {
            return Ok(());
        };

        let proxy = DistributorProxy::builder(&self.connection)
            .destination(active.distributor.clone())
            .map_err(|e| {
                io_err(format!(
                    "Invalid distributor name '{}': {e}",
                    active.distributor
                ))
            })?
            .build()
            .await
            .map_err(|e| {
                io_err(format!(
                    "Failed to connect to distributor '{}': {e}",
                    active.distributor
                ))
            })?;

        proxy
            .unregister(&active.client_token)
            .await
            .map_err(|e| io_err(format!("Distributor Unregister failed: {e}")))?;

        Ok(())
    }
}

struct ConnectorService {
    state: Weak<UnifiedPushState>,
}

#[zbus::interface(name = "org.unifiedpush.Connector1")]
impl ConnectorService {
    // `async` kept for consistency with the other D-Bus methods in this interface,
    // even though the body is currently synchronous.
    #[allow(clippy::unused_async)]
    async fn message(&self, token: String, message: Vec<u8>, id: String) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        handle_message(&state, &token, &message, &id);
    }

    async fn new_endpoint(&self, token: String, endpoint: String) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        let waiter = state.pending.lock().await.remove(&token);
        if let Some(tx) = waiter {
            let _ = tx.send(Ok(endpoint));
        }
    }

    async fn unregistered(&self, token: String) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        let mut guard = state.active.write().await;
        if guard.as_ref().is_some_and(|a| a.client_token == token) {
            *guard = None;
        }
    }

    async fn registration_failed(&self, token: String, reason: String) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        let waiter = state.pending.lock().await.remove(&token);
        if let Some(tx) = waiter {
            let _ = tx.send(Err(reason));
        }
    }
}

fn handle_message(_state: &UnifiedPushState, _token: &str, message: &[u8], _id: &str) {
    let parsed = parse_message_payload(message);

    let payload = json!({
        "source": "push",
        "title": parsed.title,
        "body": parsed.body,
        "extra": parsed.extra,
    });

    if let Err(e) = crate::listeners::trigger("notification", payload.to_string()) {
        log::warn!("Failed to dispatch push notification to listeners: {e}");
    }

    #[cfg(feature = "notify-rust")]
    show_toast(&parsed);
}

#[cfg(feature = "notify-rust")]
fn show_toast(parsed: &ParsedPayload) {
    let mut notification = notify_rust::Notification::new();
    if let Some(title) = parsed.title.as_deref() {
        notification.summary(title);
    }
    if let Some(body) = parsed.body.as_deref() {
        notification.body(body);
    }
    notification.auto_icon();
    // Leak the handle to keep the D-Bus connection alive — see the note in
    // `desktop.rs::imp::Notification::show` for the rationale.
    tauri::async_runtime::spawn_blocking(move || match notification.show() {
        Ok(handle) => {
            std::mem::forget(handle);
        }
        Err(e) => log::warn!("Failed to show push notification toast: {e}"),
    });
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ParsedPayload {
    title: Option<String>,
    body: Option<String>,
    extra: Option<JsonValue>,
}

fn parse_message_payload(bytes: &[u8]) -> ParsedPayload {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return ParsedPayload {
            extra: Some(JsonValue::String(format!("<binary {} bytes>", bytes.len()))),
            ..ParsedPayload::default()
        };
    };

    if let Ok(value) = serde_json::from_str::<JsonValue>(text) {
        let title = value
            .get("title")
            .and_then(|v| v.as_str())
            .map(String::from);
        let body = value
            .get("body")
            .or_else(|| value.get("message"))
            .and_then(|v| v.as_str())
            .map(String::from);
        let extra = value.get("data").or_else(|| value.get("extra")).cloned();
        return ParsedPayload { title, body, extra };
    }

    ParsedPayload {
        body: Some(text.to_string()),
        ..ParsedPayload::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_message_with_title_and_body() {
        let bytes = br#"{"title":"hi","body":"hello"}"#;
        let parsed = parse_message_payload(bytes);
        assert_eq!(parsed.title.as_deref(), Some("hi"));
        assert_eq!(parsed.body.as_deref(), Some("hello"));
        assert!(parsed.extra.is_none());
    }

    #[test]
    fn parse_json_message_with_data_and_message_alias() {
        let bytes = br#"{"message":"hey","data":{"k":"v"}}"#;
        let parsed = parse_message_payload(bytes);
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.body.as_deref(), Some("hey"));
        assert_eq!(parsed.extra, Some(json!({"k":"v"})));
    }

    #[test]
    fn parse_plain_text_becomes_body() {
        let bytes = b"hello there";
        let parsed = parse_message_payload(bytes);
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.body.as_deref(), Some("hello there"));
        assert_eq!(parsed.extra, None);
    }

    #[test]
    fn parse_invalid_utf8_falls_back_to_marker() {
        let bytes = &[0xff, 0xfe, 0xfd];
        let parsed = parse_message_payload(bytes);
        assert!(parsed.title.is_none());
        assert!(parsed.body.is_none());
        assert!(matches!(parsed.extra, Some(JsonValue::String(_))));
    }

    #[test]
    fn err_distributor_unavailable_includes_name() {
        let msg = err_distributor_unavailable("org.unifiedpush.Distributor.foo");
        assert!(msg.contains("org.unifiedpush.Distributor.foo"));
    }
}
