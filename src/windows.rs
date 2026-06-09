//! Windows implementation for notifications plugin using native Windows Toast API.

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, RwLock, Weak};

use nt_time::FileTime;
use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PermissionState, PluginApi},
    AppHandle, Runtime,
};
use windows::core::{implement, Interface, Ref, BOOL, GUID, HSTRING, PCWSTR};
use windows::ApplicationModel::Package;
use windows::Data::Xml::Dom::XmlDocument;
use windows::Foundation::{DateTime, TypedEventHandler};
#[cfg(feature = "push-notifications")]
use windows::Networking::PushNotifications::{
    PushNotificationChannel, PushNotificationChannelManager,
};
use windows::Win32::Foundation::{CLASS_E_NOAGGREGATION, E_INVALIDARG, S_FALSE, S_OK};
use windows::Win32::System::Com::{
    CoInitializeEx, CoRegisterClassObject, IClassFactory, IClassFactory_Impl, CLSCTX_LOCAL_SERVER,
    COINIT_APARTMENTTHREADED, REGCLS_MULTIPLEUSE,
};
use windows::Win32::UI::Notifications::{
    INotificationActivationCallback, INotificationActivationCallback_Impl,
    NOTIFICATION_USER_INPUT_DATA,
};
use windows::UI::Notifications::{
    NotificationSetting, ScheduledToastNotification, ToastActivatedEventArgs, ToastNotification,
    ToastNotificationManager, ToastNotifier,
};

use crate::error::{ErrorResponse, PluginInvokeError};
use crate::models::*;
use crate::WindowsConfig;

/// True when the current process has MSIX package identity.
///
/// WinRT notification APIs split into two flavors: no-arg variants that use the
/// package's default AUMID (only valid for packaged apps), and `*WithId`
/// variants that take an explicit AUMID (only valid for unpackaged apps whose
/// AUMID is registered via a Start Menu shortcut). Passing an arbitrary string
/// to the WithId variants from inside an MSIX returns ERROR_NOT_FOUND because
/// the real AUMID is `<PackageFamilyName>!<Application Id>` and the family-name
/// hash is install-time only.
fn is_packaged() -> bool {
    Package::Current().is_ok()
}

/// Accept any well-formed UUID string and reinterpret its bytes as a `GUID`.
///
/// Delegating to `uuid::Uuid::parse_str` lets the manifest CLSID and the
/// `tauri.conf.json` CLSID use either braced (`{xxxxxxxx-…}`), unbraced
/// (`xxxxxxxx-…`), or simple (32 hex chars, no hyphens) conventions without
/// drift causing parse failures.
fn parse_clsid(raw: &str) -> windows::core::Result<GUID> {
    let parsed = uuid::Uuid::parse_str(raw.trim())
        .map_err(|e| windows::core::Error::new(E_INVALIDARG, format!("{e}")))?;
    Ok(GUID::from_u128(parsed.as_u128()))
}

// Enable `?` operator for windows::core::Error
impl From<windows::core::Error> for crate::Error {
    fn from(err: windows::core::Error) -> Self {
        crate::Error::from(PluginInvokeError::InvokeRejected(ErrorResponse {
            code: Some(format!("0x{:08X}", err.code().0)),
            message: Some(err.message().to_string()),
            data: (),
        }))
    }
}

/// Shared plugin state wrapped in Arc for thread-safe access.
pub struct WindowsPlugin {
    app_id: String,
    packaged: bool,
    notifier: ToastNotifier,
    action_types: RwLock<HashMap<String, ActionType>>,
    click_listener_active: RwLock<bool>,
    /// Cold-start activation payloads queued before any JS listener has
    /// subscribed. Drained synchronously the first time a `notificationClicked`
    /// listener registers (see `crate::listeners::register_listener`).
    pending_clicks: RwLock<Vec<serde_json::Value>>,
    /// `CoRegisterClassObject` cookie. Kept for the process lifetime — no
    /// explicit `CoRevokeClassObject` on shutdown; the OS reclaims it on exit.
    /// `None` when COM activator wasn't registered (unpackaged or no CLSID in
    /// config).
    _com_cookie: RwLock<Option<u32>>,
    #[cfg(feature = "push-notifications")]
    push_channel: RwLock<Option<PushNotificationChannel>>,
}

/// COM activator that receives toast activations from Action Center, including
/// the cold-start case where Windows launches the exe via the manifest's
/// `windows.toastNotificationActivation` extension.
///
/// Wired up by `init()` only when the process has MSIX package identity AND
/// the plugin config carries a valid `toast_activator_clsid`. The callback
/// fires on a COM RPC worker thread (not the Tauri main thread), so all
/// downstream emissions must be thread-safe — `crate::listeners::trigger`
/// already is.
#[implement(INotificationActivationCallback)]
struct ToastActivator {
    plugin: Weak<WindowsPlugin>,
}

/// Out-of-proc COM activator pattern requires a class factory; `CoRegisterClassObject`
/// takes an `IUnknown` that must implement `IClassFactory`, not the activator
/// instance directly. There is no shortcut for the toast activator path.
#[implement(IClassFactory)]
struct ToastActivatorFactory {
    plugin: Weak<WindowsPlugin>,
}

impl std::fmt::Debug for WindowsPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsPlugin")
            .field("app_id", &self.app_id)
            .field("packaged", &self.packaged)
            .finish_non_exhaustive()
    }
}

/// Build the `actionPerformed` payload for the COM activator (cold-start /
/// Action Center activations). The warm in-process `ToastNotification.Activated`
/// handler emits the same shape but populates `notification` with the originating
/// `ActiveNotification`; the cold path can't because Windows hands the activator
/// only `invokedArgs` + text inputs, with no reference back to the originating
/// notification. Consumers that need the notification body should treat
/// `notification == null` as the cold-start signal.
fn activation_payload(invoked_args: &str, inputs: &HashMap<String, String>) -> serde_json::Value {
    let action_id = if invoked_args.is_empty() {
        "tap"
    } else {
        invoked_args
    };
    let input_value = inputs
        .values()
        .next()
        .cloned()
        .map_or(serde_json::Value::Null, serde_json::Value::String);
    serde_json::json!({
        "actionId": action_id,
        "inputValue": input_value,
        "notification": serde_json::Value::Null,
    })
}

impl INotificationActivationCallback_Impl for ToastActivator_Impl {
    fn Activate(
        &self,
        _appusermodelid: &PCWSTR,
        invokedargs: &PCWSTR,
        data: *const NOTIFICATION_USER_INPUT_DATA,
        count: u32,
    ) -> windows::core::Result<()> {
        let invoked = unsafe { invokedargs.to_string() }.unwrap_or_default();
        let mut inputs: HashMap<String, String> = HashMap::new();
        if count > 0 && !data.is_null() {
            let slice = unsafe { std::slice::from_raw_parts(data, count as usize) };
            for entry in slice {
                let k = unsafe { entry.Key.to_string() }.unwrap_or_default();
                let v = unsafe { entry.Value.to_string() }.unwrap_or_default();
                if !k.is_empty() {
                    inputs.insert(k, v);
                }
            }
        }

        let action_payload = activation_payload(&invoked, &inputs);
        let _ = crate::listeners::trigger("actionPerformed", action_payload.to_string());

        if invoked.is_empty() {
            // Cold-start payload shape differs from the in-process Activated path:
            // here `id`/`data` are null/empty because Windows hands the activator
            // no reference back to the originating notification. Consumers can
            // distinguish via `id == null`.
            let click_payload = serde_json::json!({ "id": serde_json::Value::Null, "data": {} });

            // Deliver live OR buffer — never both. Buffering when a listener is
            // already subscribed causes duplicate events on the next re-subscribe
            // (hot reload, route change).
            if crate::listeners::has_listeners("notificationClicked") {
                let _ = crate::listeners::trigger("notificationClicked", click_payload.to_string());
            } else if let Some(plugin) = self.plugin.upgrade() {
                if let Ok(mut buf) = plugin.pending_clicks.write() {
                    buf.push(click_payload);
                }
            }
        }
        Ok(())
    }
}

impl IClassFactory_Impl for ToastActivatorFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, windows::core::IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut c_void,
    ) -> windows::core::Result<()> {
        if !punkouter.is_null() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let activator = ToastActivator {
            plugin: self.plugin.clone(),
        };
        let interface: INotificationActivationCallback = activator.into();
        unsafe { interface.query(riid, ppvobject).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> windows::core::Result<()> {
        Ok(())
    }
}

impl WindowsPlugin {
    fn action_types(&self) -> crate::Result<HashMap<String, ActionType>> {
        Ok(self
            .action_types
            .read()
            .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))?
            .clone())
    }

    fn action_types_mut(
        &self,
    ) -> crate::Result<std::sync::RwLockWriteGuard<'_, HashMap<String, ActionType>>> {
        self.action_types
            .write()
            .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))
    }

    fn is_click_listener_active(&self) -> crate::Result<bool> {
        Ok(*self
            .click_listener_active
            .read()
            .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))?)
    }

    fn set_click_listener(&self, active: bool) -> crate::Result<()> {
        *self
            .click_listener_active
            .write()
            .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))? = active;
        Ok(())
    }

    /// Drain queued cold-start click payloads through the listener bus. Called
    /// when a `notificationClicked` listener subscribes (see
    /// `crate::listeners::register_listener`). Idempotent: subsequent calls
    /// with an empty buffer are a no-op.
    pub fn drain_pending_clicks(&self) {
        let drained: Vec<serde_json::Value> = match self.pending_clicks.write() {
            Ok(mut buf) => std::mem::take(&mut *buf),
            Err(e) => {
                log::error!("pending_clicks lock poisoned during drain: {e}");
                return;
            }
        };
        for payload in drained {
            if let Err(e) = crate::listeners::trigger("notificationClicked", payload.to_string()) {
                log::error!("Failed to dispatch buffered click: {e}");
            }
        }
    }

    fn open_push_channel(&self) -> crate::Result<String> {
        #[cfg(feature = "push-notifications")]
        {
            let channel =
                PushNotificationChannelManager::CreatePushNotificationChannelForApplicationAsync()?
                    .get()?;
            let uri = channel.Uri()?.to_string_lossy();
            *self
                .push_channel
                .write()
                .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))? =
                Some(channel);
            Ok(uri)
        }
        #[cfg(not(feature = "push-notifications"))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Push notifications feature not enabled",
            )))
        }
    }

    fn close_push_channel(&self) -> crate::Result<()> {
        #[cfg(feature = "push-notifications")]
        {
            if let Some(channel) = self
                .push_channel
                .write()
                .map_err(|_| crate::Error::Io(std::io::Error::other("Lock poisoned")))?
                .take()
            {
                channel.Close()?;
            }
            Ok(())
        }
        #[cfg(not(feature = "push-notifications"))]
        {
            Err(crate::Error::Io(std::io::Error::other(
                "Push notifications feature not enabled",
            )))
        }
    }
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
    windows_config: WindowsConfig,
) -> crate::Result<Notifications<R>> {
    let app_id = app.config().identifier.clone();
    let packaged = is_packaged();
    let notifier = if packaged {
        ToastNotificationManager::CreateToastNotifier()?
    } else {
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(&app_id))?
    };

    let plugin = Arc::new(WindowsPlugin {
        app_id,
        packaged,
        notifier,
        action_types: RwLock::new(HashMap::new()),
        click_listener_active: RwLock::new(false),
        pending_clicks: RwLock::new(Vec::new()),
        _com_cookie: RwLock::new(None),
        #[cfg(feature = "push-notifications")]
        push_channel: RwLock::new(None),
    });

    if packaged {
        if let Some(clsid_str) = windows_config.toast_activator_clsid.as_deref() {
            match register_toast_activator(&plugin, clsid_str) {
                Ok(cookie) => {
                    if let Ok(mut slot) = plugin._com_cookie.write() {
                        *slot = Some(cookie);
                    }
                    log::info!("Toast activator registered (clsid={clsid_str}, cookie={cookie})");
                }
                Err(e) => {
                    log::error!(
                        "Failed to register toast activator (clsid={clsid_str}): {e}; \
                         Action Center clicks will fall back to shortcut launch without payload"
                    );
                }
            }
        }
    }

    Ok(Notifications {
        app: app.clone(),
        plugin,
    })
}

/// Initialize COM for the current thread (apartment-threaded) and register a
/// `ToastActivatorFactory` for the given CLSID. Returns the
/// `CoRegisterClassObject` cookie on success.
///
/// `CoInitializeEx` is idempotent for the same apartment model: `S_OK` on
/// first call, `S_FALSE` on subsequent calls. `RPC_E_CHANGED_MODE` means
/// another component already initialized this thread as MTA — surfaced as an
/// error so the caller logs and skips registration.
fn register_toast_activator(
    plugin: &Arc<WindowsPlugin>,
    clsid_str: &str,
) -> windows::core::Result<u32> {
    let clsid = parse_clsid(clsid_str)?;
    let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if hr != S_OK && hr != S_FALSE {
        return Err(hr.into());
    }
    let factory = ToastActivatorFactory {
        plugin: Arc::downgrade(plugin),
    };
    let factory_interface: IClassFactory = factory.into();
    unsafe {
        CoRegisterClassObject(
            &clsid,
            &factory_interface,
            CLSCTX_LOCAL_SERVER,
            REGCLS_MULTIPLEUSE,
        )
    }
}

impl<R: Runtime> crate::NotificationsBuilder<R> {
    /// Build toast notification XML using DOM API (safer than string concatenation).
    fn build_toast_xml(
        &self,
        action_types: &HashMap<String, ActionType>,
    ) -> crate::Result<XmlDocument> {
        let doc = XmlDocument::new()?;

        // Create root <toast>
        let toast = doc.CreateElement(&HSTRING::from("toast"))?;
        doc.AppendChild(&toast)?;

        // Create <visual><binding template="ToastGeneric">
        let visual = doc.CreateElement(&HSTRING::from("visual"))?;
        let binding = doc.CreateElement(&HSTRING::from("binding"))?;
        binding.SetAttribute(&HSTRING::from("template"), &HSTRING::from("ToastGeneric"))?;

        // Add <text> elements for title/body
        if let Some(title) = &self.data.title {
            let text = doc.CreateElement(&HSTRING::from("text"))?;
            text.SetInnerText(&HSTRING::from(title.as_str()))?;
            binding.AppendChild(&text)?;
        }

        if let Some(body) = &self.data.body {
            let text = doc.CreateElement(&HSTRING::from("text"))?;
            text.SetInnerText(&HSTRING::from(body.as_str()))?;
            binding.AppendChild(&text)?;
        }

        if let Some(large_body) = &self.data.large_body {
            let text = doc.CreateElement(&HSTRING::from("text"))?;
            text.SetInnerText(&HSTRING::from(large_body.as_str()))?;
            binding.AppendChild(&text)?;
        }

        // Add icon if specified
        if let Some(icon) = &self.data.icon {
            let image = doc.CreateElement(&HSTRING::from("image"))?;
            image.SetAttribute(
                &HSTRING::from("placement"),
                &HSTRING::from("appLogoOverride"),
            )?;
            image.SetAttribute(&HSTRING::from("src"), &HSTRING::from(icon.as_str()))?;
            binding.AppendChild(&image)?;
        }

        // Add attachments as images
        for (i, attachment) in self.data.attachments.iter().enumerate() {
            let image = doc.CreateElement(&HSTRING::from("image"))?;
            // First attachment as hero image, rest as inline
            if i == 0 {
                image.SetAttribute(&HSTRING::from("placement"), &HSTRING::from("hero"))?;
            }
            image.SetAttribute(
                &HSTRING::from("src"),
                &HSTRING::from(attachment.url().as_str()),
            )?;
            binding.AppendChild(&image)?;
        }

        visual.AppendChild(&binding)?;
        toast.AppendChild(&visual)?;

        // Add <actions> if action_type_id specified
        if let Some(action_type_id) = &self.data.action_type_id {
            if let Some(action_type) = action_types.get(action_type_id) {
                let actions = doc.CreateElement(&HSTRING::from("actions"))?;
                for action in action_type.actions() {
                    let action_el = doc.CreateElement(&HSTRING::from("action"))?;
                    action_el
                        .SetAttribute(&HSTRING::from("content"), &HSTRING::from(action.title()))?;
                    action_el
                        .SetAttribute(&HSTRING::from("arguments"), &HSTRING::from(action.id()))?;
                    let activation_type = if action.foreground() {
                        "foreground"
                    } else {
                        "background"
                    };
                    action_el.SetAttribute(
                        &HSTRING::from("activationType"),
                        &HSTRING::from(activation_type),
                    )?;
                    actions.AppendChild(&action_el)?;
                }
                toast.AppendChild(&actions)?;
            }
        }

        // Add <audio> element for silent or custom sound
        if self.data.silent {
            let audio = doc.CreateElement(&HSTRING::from("audio"))?;
            audio.SetAttribute(&HSTRING::from("silent"), &HSTRING::from("true"))?;
            toast.AppendChild(&audio)?;
        } else if let Some(sound) = &self.data.sound {
            let audio = doc.CreateElement(&HSTRING::from("audio"))?;
            audio.SetAttribute(&HSTRING::from("src"), &HSTRING::from(sound.as_str()))?;
            toast.AppendChild(&audio)?;
        }

        Ok(doc)
    }

    pub async fn show(self) -> crate::Result<()> {
        let action_types = self.plugin.action_types()?;
        let toast_xml = self.build_toast_xml(&action_types)?;

        let tag = HSTRING::from(self.data.id.to_string());
        let group = self.data.group.as_ref().map(|g| HSTRING::from(g.as_str()));

        // Check if this is a scheduled notification
        if let Some(schedule) = &self.data.schedule {
            let delivery_time = schedule_to_datetime(schedule)?;
            let scheduled = ScheduledToastNotification::CreateScheduledToastNotification(
                &toast_xml,
                delivery_time,
            )?;

            scheduled.SetTag(&tag)?;
            if let Some(g) = &group {
                scheduled.SetGroup(g)?;
            }

            self.plugin.notifier.AddToSchedule(&scheduled)?;
        } else {
            // Immediate notification
            let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
            toast.SetTag(&tag)?;
            if let Some(g) = &group {
                toast.SetGroup(g)?;
            }

            if self.plugin.is_click_listener_active()? {
                let notification = ActiveNotification {
                    id: self.data.id,
                    tag: Some(self.data.id.to_string()),
                    title: self.data.title.clone(),
                    body: self.data.body.clone(),
                    group: self.data.group.clone(),
                    group_summary: self.data.group_summary,
                    data: HashMap::new(),
                    extra: self.data.extra.clone(),
                    attachments: self.data.attachments.clone(),
                    action_type_id: self.data.action_type_id.clone(),
                    schedule: self.data.schedule.clone(),
                    sound: self.data.sound.clone(),
                };

                toast.Activated(&TypedEventHandler::new(
                    move |_: windows::core::Ref<'_, ToastNotification>,
                          args: windows::core::Ref<'_, windows::core::IInspectable>| {
                        if let Some(inspectable) = &*args {
                            if let Ok(activated) = inspectable.cast::<ToastActivatedEventArgs>() {
                                let arguments = activated
                                    .Arguments()
                                    .map(|s| s.to_string_lossy())
                                    .unwrap_or_default();

                                let action_id = if arguments.is_empty() {
                                    "tap".to_string()
                                } else {
                                    arguments.to_string()
                                };

                                let payload = serde_json::json!({
                                    "actionId": action_id,
                                    "inputValue": null,
                                    "notification": notification,
                                });
                                if let Err(e) = crate::listeners::trigger(
                                    "actionPerformed",
                                    payload.to_string(),
                                ) {
                                    log::error!("Failed to trigger actionPerformed: {e}");
                                }

                                if arguments.is_empty() {
                                    let click_payload = serde_json::json!({
                                        "id": notification.id,
                                        "data": notification.extra,
                                    });
                                    if let Err(e) = crate::listeners::trigger(
                                        "notificationClicked",
                                        click_payload.to_string(),
                                    ) {
                                        log::error!("Failed to trigger notificationClicked: {e}");
                                    }
                                }
                            }
                        }
                        Ok(())
                    },
                ))?;
            }

            self.plugin.notifier.Show(&toast)?;
        }

        // Trigger notification event
        let payload = serde_json::json!({
            "id": self.data.id,
            "title": self.data.title,
            "body": self.data.body,
            "actionTypeId": self.data.action_type_id,
            "extra": self.data.extra,
        });
        if let Err(e) = crate::listeners::trigger("notification", payload.to_string()) {
            log::error!("Failed to trigger notification: {e}");
        }

        Ok(())
    }
}

/// Convert Schedule to Windows DateTime.
fn schedule_to_datetime(schedule: &Schedule) -> crate::Result<DateTime> {
    let now = time::OffsetDateTime::now_utc();

    let delivery_time = match schedule {
        Schedule::At { date, .. } => *date,
        Schedule::Interval { interval, .. } => {
            // Build duration from interval fields
            let seconds = interval.second.unwrap_or(0) as i64;
            let minutes = interval.minute.unwrap_or(0) as i64;
            let hours = interval.hour.unwrap_or(0) as i64;
            let days = interval.day.unwrap_or(0) as i64;
            let total_seconds = seconds + minutes * 60 + hours * 3600 + days * 86400;
            now + time::Duration::seconds(total_seconds)
        }
        Schedule::Every {
            interval, count, ..
        } => {
            let base_seconds: i64 = match interval {
                ScheduleEvery::Year => 365 * 86400,
                ScheduleEvery::Month => 30 * 86400,
                ScheduleEvery::TwoWeeks => 14 * 86400,
                ScheduleEvery::Week => 7 * 86400,
                ScheduleEvery::Day => 86400,
                ScheduleEvery::Hour => 3600,
                ScheduleEvery::Minute => 60,
                ScheduleEvery::Second => 1,
            };
            now + time::Duration::seconds(base_seconds * (*count as i64))
        }
    };

    unix_to_windows_datetime(delivery_time)
}

/// Convert a Unix timestamp to Windows DateTime (FILETIME).
fn unix_to_windows_datetime(time: time::OffsetDateTime) -> crate::Result<DateTime> {
    let ft = FileTime::try_from(time.to_utc())
        .map_err(|_| crate::Error::Io(std::io::Error::other("Schedule date out of range")))?;
    let raw: i64 = ft
        .to_raw()
        .try_into()
        .map_err(|_| crate::Error::Io(std::io::Error::other("Schedule date out of range")))?;
    Ok(DateTime { UniversalTime: raw })
}

/// Convert Windows DateTime (FILETIME) back to Unix timestamp.
fn windows_datetime_to_unix(dt: DateTime) -> crate::Result<time::OffsetDateTime> {
    let raw: u64 = dt
        .UniversalTime
        .try_into()
        .map_err(|_| crate::Error::Io(std::io::Error::other("DateTime out of range")))?;
    let utc = time::UtcDateTime::try_from(FileTime::new(raw))
        .map_err(|_| crate::Error::Io(std::io::Error::other("DateTime out of range")))?;
    Ok(utc.into())
}

pub struct Notifications<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    plugin: Arc<WindowsPlugin>,
}

impl<R: Runtime> Notifications<R> {
    pub fn builder(&self) -> crate::NotificationsBuilder<R> {
        crate::NotificationsBuilder::new(self.app.clone(), self.plugin.clone())
    }

    /// Drain any cold-start activation payloads queued before the JS
    /// `notificationClicked` listener subscribed. Invoked by
    /// `crate::listeners::register_listener` on first subscription so the
    /// `push-listener.tsx` contract ("subscribing flushes the buffered tap")
    /// holds without the app having to call any extra command.
    pub fn drain_pending_clicks(&self) {
        self.plugin.drain_pending_clicks();
    }

    pub async fn request_permission(&self) -> crate::Result<PermissionState> {
        // Windows doesn't have a runtime permission prompt like mobile
        // We can only check the current state
        self.permission_state().await
    }

    pub async fn register_for_push_notifications(&self) -> crate::Result<String> {
        self.plugin.open_push_channel()
    }

    pub fn unregister_for_push_notifications(&self) -> crate::Result<()> {
        self.plugin.close_push_channel()
    }

    pub async fn permission_state(&self) -> crate::Result<PermissionState> {
        match self.plugin.notifier.Setting()? {
            NotificationSetting::Enabled => Ok(PermissionState::Granted),
            NotificationSetting::DisabledForApplication
            | NotificationSetting::DisabledForUser
            | NotificationSetting::DisabledByGroupPolicy
            | NotificationSetting::DisabledByManifest => Ok(PermissionState::Denied),
            _ => Ok(PermissionState::Prompt),
        }
    }

    pub fn register_action_types(&self, types: Vec<ActionType>) -> crate::Result<()> {
        let mut action_types = self.plugin.action_types_mut()?;
        for action_type in types {
            action_types.insert(action_type.id().to_string(), action_type);
        }
        Ok(())
    }

    pub fn remove_active(&self, notifications: Vec<i32>) -> crate::Result<()> {
        let history = ToastNotificationManager::History()?;
        let app_id = &self.plugin.app_id;
        for id in notifications {
            let tag = HSTRING::from(id.to_string());
            // Use app-scoped removal with empty group (consistent with GetHistoryWithId usage)
            let res = if self.plugin.packaged {
                history.RemoveGroupedTag(&tag, &HSTRING::new())
            } else {
                history.RemoveGroupedTagWithId(&tag, &HSTRING::new(), &HSTRING::from(app_id))
            };
            if let Err(e) = res {
                log::error!("Failed to remove notification {id}: {e}");
            }
        }
        Ok(())
    }

    pub async fn active(&self) -> crate::Result<Vec<ActiveNotification>> {
        let history = ToastNotificationManager::History()?;
        let notifications = if self.plugin.packaged {
            history.GetHistory()?
        } else {
            history.GetHistoryWithId(&HSTRING::from(&self.plugin.app_id))?
        };

        let mut result = Vec::new();
        for i in 0..notifications.Size()? {
            let notification = notifications.GetAt(i)?;
            let tag = notification.Tag()?.to_string_lossy();
            let id = tag.parse::<i32>().unwrap_or(0);
            let group = notification.Group().ok().map(|s| s.to_string_lossy());

            // Extract title/body from XML content
            let (title, body) = if let Ok(content) = notification.Content() {
                let text_elements = content.GetElementsByTagName(&HSTRING::from("text"))?;
                let title = text_elements
                    .GetAt(0)
                    .ok()
                    .and_then(|el| el.InnerText().ok())
                    .map(|s| s.to_string_lossy());
                let body = text_elements
                    .GetAt(1)
                    .ok()
                    .and_then(|el| el.InnerText().ok())
                    .map(|s| s.to_string_lossy());
                (title, body)
            } else {
                (None, None)
            };

            result.push(ActiveNotification {
                id,
                tag: Some(tag),
                title,
                body,
                group,
                group_summary: false,
                data: HashMap::new(),
                extra: HashMap::new(),
                attachments: Vec::new(),
                action_type_id: None,
                schedule: None,
                sound: None,
            });
        }

        Ok(result)
    }

    pub fn remove_all_active(&self) -> crate::Result<()> {
        let history = ToastNotificationManager::History()?;
        if self.plugin.packaged {
            history.Clear()?;
        } else {
            history.ClearWithId(&HSTRING::from(&self.plugin.app_id))?;
        }
        Ok(())
    }

    pub async fn pending(&self) -> crate::Result<Vec<PendingNotification>> {
        let scheduled = self.plugin.notifier.GetScheduledToastNotifications()?;
        let mut result = Vec::new();

        for i in 0..scheduled.Size()? {
            let notification = scheduled.GetAt(i)?;
            let tag = notification.Tag()?.to_string_lossy();
            let id = tag.parse::<i32>().unwrap_or(0);

            let (title, body) = if let Ok(content) = notification.Content() {
                let text_elements = content.GetElementsByTagName(&HSTRING::from("text"))?;
                let title = text_elements
                    .GetAt(0)
                    .ok()
                    .and_then(|el| el.InnerText().ok())
                    .map(|s| s.to_string_lossy());
                let body = text_elements
                    .GetAt(1)
                    .ok()
                    .and_then(|el| el.InnerText().ok())
                    .map(|s| s.to_string_lossy());
                (title, body)
            } else {
                (None, None)
            };

            // Convert Windows DateTime back to Schedule::At
            let schedule = notification.DeliveryTime().ok().and_then(|dt| {
                windows_datetime_to_unix(dt).ok().map(|date| Schedule::At {
                    date,
                    repeating: false,
                    allow_while_idle: false,
                })
            });

            // PendingNotification requires schedule (not Option), skip if we can't extract it
            if let Some(schedule) = schedule {
                result.push(PendingNotification {
                    id,
                    title,
                    body,
                    schedule,
                });
            }
        }

        Ok(result)
    }

    pub fn cancel(&self, notifications: Vec<i32>) -> crate::Result<()> {
        let scheduled = self.plugin.notifier.GetScheduledToastNotifications()?;
        let ids_to_cancel: std::collections::HashSet<_> = notifications.into_iter().collect();

        for i in 0..scheduled.Size()? {
            if let Ok(notification) = scheduled.GetAt(i) {
                if let Ok(tag) = notification.Tag() {
                    if let Ok(id) = tag.to_string_lossy().parse::<i32>() {
                        if ids_to_cancel.contains(&id) {
                            if let Err(e) = self.plugin.notifier.RemoveFromSchedule(&notification) {
                                log::error!("Failed to cancel notification {id}: {e}");
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn cancel_all(&self) -> crate::Result<()> {
        let scheduled = self.plugin.notifier.GetScheduledToastNotifications()?;
        for i in 0..scheduled.Size()? {
            if let Ok(notification) = scheduled.GetAt(i) {
                if let Err(e) = self.plugin.notifier.RemoveFromSchedule(&notification) {
                    log::error!("Failed to cancel scheduled notification: {e}");
                }
            }
        }
        Ok(())
    }

    pub fn set_click_listener_active(&self, active: bool) -> crate::Result<()> {
        self.plugin.set_click_listener(active)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// PowerShell App User Model ID - always available on Windows.
    const POWERSHELL_APP_ID: &str =
        "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";

    // ==================== Time Conversion Tests ====================

    /// Windows FILETIME epoch (1601-01-01) offset from Unix epoch (1970-01-01),
    /// in 100-nanosecond ticks. Used only as a test reference value.
    const WINDOWS_EPOCH_OFFSET_TICKS: i128 = 116_444_736_000_000_000;

    #[test]
    fn test_unix_to_windows_datetime_epoch() {
        let result = unix_to_windows_datetime(time::OffsetDateTime::UNIX_EPOCH)
            .expect("Failed to convert Unix epoch");
        assert_eq!(result.UniversalTime as i128, WINDOWS_EPOCH_OFFSET_TICKS);
    }

    #[test]
    fn test_unix_to_windows_datetime_known_date() {
        let date = time::macros::datetime!(2000-01-01 00:00:00 UTC);
        let result = unix_to_windows_datetime(date).expect("Failed to convert known date");

        let unix_nanos = 946_684_800i128 * 1_000_000_000;
        let expected = (unix_nanos / 100) + WINDOWS_EPOCH_OFFSET_TICKS;
        assert_eq!(result.UniversalTime as i128, expected);
    }

    #[test]
    fn test_windows_datetime_roundtrip() {
        let original = time::macros::datetime!(2024-06-15 14:30:45 UTC);
        let windows_dt =
            unix_to_windows_datetime(original).expect("Failed to convert to Windows datetime");
        let roundtrip =
            windows_datetime_to_unix(windows_dt).expect("Failed to convert back to Unix");

        let diff = (original - roundtrip).whole_nanoseconds().abs();
        assert!(diff < 100, "Roundtrip diff: {}ns", diff);
    }

    #[test]
    fn test_schedule_at_conversion() {
        let target = time::macros::datetime!(2025-12-25 10:00:00 UTC);
        let schedule = Schedule::At {
            date: target,
            repeating: false,
            allow_while_idle: false,
        };

        let result = schedule_to_datetime(&schedule).expect("Failed to convert schedule");
        let back = windows_datetime_to_unix(result).expect("Failed to convert back");
        assert!((target - back).whole_nanoseconds().abs() < 100);
    }

    #[test]
    fn test_schedule_interval() {
        let schedule = Schedule::Interval {
            interval: ScheduleInterval {
                year: None,
                month: None,
                day: Some(1),
                weekday: None,
                hour: Some(2),
                minute: Some(30),
                second: Some(45),
            },
            allow_while_idle: false,
        };

        let before = time::OffsetDateTime::now_utc();
        let result = schedule_to_datetime(&schedule).expect("Failed to convert interval schedule");
        let converted = windows_datetime_to_unix(result).expect("Failed to convert back");

        let expected = 86400 + 7200 + 1800 + 45; // 1d + 2h + 30m + 45s
        let actual = (converted - before).whole_seconds();
        assert!((actual - expected).abs() <= 2);
    }

    #[test]
    fn test_schedule_every_variants() {
        let cases = [
            (ScheduleEvery::Second, 1, 1i64),
            (ScheduleEvery::Minute, 1, 60),
            (ScheduleEvery::Hour, 1, 3600),
            (ScheduleEvery::Day, 1, 86400),
            (ScheduleEvery::Week, 1, 7 * 86400),
            (ScheduleEvery::TwoWeeks, 1, 14 * 86400),
            (ScheduleEvery::Month, 1, 30 * 86400),
            (ScheduleEvery::Year, 1, 365 * 86400),
        ];

        for (interval, count, expected) in cases {
            let schedule = Schedule::Every {
                interval,
                count,
                allow_while_idle: false,
            };

            let before = time::OffsetDateTime::now_utc();
            let result = schedule_to_datetime(&schedule)
                .unwrap_or_else(|e| panic!("Failed to convert {:?}: {}", interval, e));
            let converted = windows_datetime_to_unix(result)
                .unwrap_or_else(|e| panic!("Failed to convert back {:?}: {}", interval, e));
            let actual = (converted - before).whole_seconds();
            assert!(
                (actual - expected).abs() <= 2,
                "{:?}: {} vs {}",
                interval,
                actual,
                expected
            );
        }
    }

    // ==================== Toast Notifier Tests ====================

    #[test]
    fn test_toast_notifier_creation() {
        let result =
            ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(POWERSHELL_APP_ID));
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    // ==================== XML Building Tests ====================

    #[test]
    fn test_xml_document_creation() {
        assert!(XmlDocument::new().is_ok());
    }

    #[test]
    fn test_toast_xml_structure() {
        let doc = XmlDocument::new().expect("Failed to create XmlDocument");

        let toast = doc
            .CreateElement(&HSTRING::from("toast"))
            .expect("Failed to create toast element");
        doc.AppendChild(&toast).expect("Failed to append toast");

        let visual = doc
            .CreateElement(&HSTRING::from("visual"))
            .expect("Failed to create visual element");
        let binding = doc
            .CreateElement(&HSTRING::from("binding"))
            .expect("Failed to create binding element");
        binding
            .SetAttribute(&HSTRING::from("template"), &HSTRING::from("ToastGeneric"))
            .expect("Failed to set template attribute");

        let text = doc
            .CreateElement(&HSTRING::from("text"))
            .expect("Failed to create text element");
        text.SetInnerText(&HSTRING::from("Test Title"))
            .expect("Failed to set text content");
        binding
            .AppendChild(&text)
            .expect("Failed to append text to binding");
        visual
            .AppendChild(&binding)
            .expect("Failed to append binding to visual");
        toast
            .AppendChild(&visual)
            .expect("Failed to append visual to toast");

        let xml = doc.GetXml().expect("Failed to get XML").to_string_lossy();
        assert!(
            xml.contains("toast") && xml.contains("ToastGeneric") && xml.contains("Test Title")
        );
    }

    #[test]
    fn test_toast_xml_with_actions() {
        let doc = XmlDocument::new().expect("Failed to create XmlDocument");
        let toast = doc
            .CreateElement(&HSTRING::from("toast"))
            .expect("Failed to create toast element");
        doc.AppendChild(&toast).expect("Failed to append toast");

        let actions = doc
            .CreateElement(&HSTRING::from("actions"))
            .expect("Failed to create actions element");
        let action = doc
            .CreateElement(&HSTRING::from("action"))
            .expect("Failed to create action element");
        action
            .SetAttribute(&HSTRING::from("content"), &HSTRING::from("Accept"))
            .expect("Failed to set content attribute");
        action
            .SetAttribute(&HSTRING::from("arguments"), &HSTRING::from("accept"))
            .expect("Failed to set arguments attribute");
        actions
            .AppendChild(&action)
            .expect("Failed to append action");
        toast
            .AppendChild(&actions)
            .expect("Failed to append actions");

        let xml = doc.GetXml().expect("Failed to get XML").to_string_lossy();
        assert!(xml.contains("actions") && xml.contains("Accept"));
    }

    #[test]
    fn test_toast_xml_silent() {
        let doc = XmlDocument::new().expect("Failed to create XmlDocument");
        let toast = doc
            .CreateElement(&HSTRING::from("toast"))
            .expect("Failed to create toast element");
        doc.AppendChild(&toast).expect("Failed to append toast");

        let audio = doc
            .CreateElement(&HSTRING::from("audio"))
            .expect("Failed to create audio element");
        audio
            .SetAttribute(&HSTRING::from("silent"), &HSTRING::from("true"))
            .expect("Failed to set silent attribute");
        toast.AppendChild(&audio).expect("Failed to append audio");

        assert!(doc
            .GetXml()
            .expect("Failed to get XML")
            .to_string_lossy()
            .contains("silent"));
    }

    // ==================== Action Types Tests ====================

    #[test]
    fn test_action_types_storage() {
        let types: RwLock<HashMap<String, ActionType>> = RwLock::new(HashMap::new());
        let action_type = ActionType::new("test", vec![Action::new("btn", "Button", false)]);

        types
            .write()
            .expect("RwLock poisoned")
            .insert("test".to_string(), action_type);

        let read = types.read().expect("RwLock poisoned");
        assert!(read.contains_key("test"));
        assert_eq!(read.get("test").expect("Key not found").actions().len(), 1);
    }

    #[test]
    fn test_multiple_action_types() {
        let types: RwLock<HashMap<String, ActionType>> = RwLock::new(HashMap::new());

        {
            let mut w = types.write().expect("RwLock poisoned");
            w.insert(
                "confirm".to_string(),
                ActionType::new(
                    "confirm",
                    vec![
                        Action::new("yes", "Yes", true),
                        Action::new("no", "No", false),
                    ],
                ),
            );
            w.insert(
                "reply".to_string(),
                ActionType::new("reply", vec![Action::new("reply", "Reply", true)]),
            );
        }

        let r = types.read().expect("RwLock poisoned");
        assert_eq!(r.len(), 2);
        assert!(r.contains_key("confirm") && r.contains_key("reply"));
    }
}
