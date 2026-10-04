//! Pure decision logic for the Windows toast backend.
//!
//! `src/windows.rs` only compiles on Windows, so anything tested from inside
//! it never runs on a Linux or macOS host. The parts of the backend that are
//! plain data — which `<input>`/`<action>` elements a toast needs, how a
//! button's `arguments=` is encoded and decoded, and which `Setting()` failure
//! means "the user was never asked" — live here instead, free of any `windows`
//! crate type. `lib.rs` compiles this module under `cfg(test)` on every
//! platform so its tests run wherever `cargo test` does.

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};
use tauri::plugin::PermissionState;

use crate::models::Action;

/// Key that marks an activation's `arguments=` as coming from a button rather
/// than from the toast-level `launch=`. Absent from `launch=`, which is what
/// lets `decode_activation` tell the two apart.
const ACTION_KEY: &str = "action";

/// Carried alongside the notification id so a cold activation can rebuild the
/// same [`ClaimKey`] the in-process handler registered. Stripped back out on
/// decode, so neither payload the JS layer sees gains a field.
const GROUP_KEY: &str = "group";

/// Windows rejects an `<actions>` block with more than five of either.
const MAX_INPUTS: usize = 5;
const MAX_BUTTONS: usize = 5;

/// `actionId` reported for an activation of the toast body.
const TAP_ACTION_ID: &str = "tap";

/// An `<input type="text">` element to declare inside `<actions>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastInput {
    /// `id=`. Also the key Windows files the typed text under in the
    /// activation's user-input map, and the value of the submitting button's
    /// `hint-inputId=`. Named after the action so all three line up.
    pub id: String,
    /// `placeHolderContent=`, omitted when the action declared none.
    pub placeholder: Option<String>,
}

/// An `<action>` element — one toast button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastButton {
    /// `content=`, the button caption.
    pub content: String,
    /// `arguments=`, what Windows hands back on activation.
    pub arguments: String,
    /// `activationType=`.
    pub activation_type: &'static str,
    /// `hint-inputId=`, set only for a button that submits a text box.
    pub hint_input_id: Option<String>,
}

/// The `<actions>` block of a toast, as data.
///
/// Inputs are kept in their own list because a `hint-inputId` may only name an
/// id that already exists in the document: emitting every `<input>` before any
/// `<action>` is a requirement of the toast schema, not a style choice, so the
/// shape enforces it rather than relying on the caller's loop order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToastActions {
    pub inputs: Vec<ToastInput>,
    pub buttons: Vec<ToastButton>,
}

/// Turn a registered action type's actions into the `<actions>` block.
///
/// `launch` is the toast-level `{"id": …, "data": …}` object; it is folded
/// into every button's `arguments=` by `encode_action_arguments`.
pub fn plan_toast_actions(actions: &[Action], launch: &Value) -> ToastActions {
    let mut planned = ToastActions::default();
    // A repeated `<input id>` is rejected the same way a sixth element is, and
    // is easy to write by accident, so two actions sharing an id share one box
    // — their `hint-inputId` both point at it, which is what submitting the
    // same box from two buttons means.
    let mut declared: HashSet<&str> = HashSet::new();
    for action in actions {
        if planned.buttons.len() == MAX_BUTTONS {
            log::warn!(
                "Toast action type declares more than {MAX_BUTTONS} actions; Windows rejects \
                 the whole toast past that, so the rest are dropped"
            );
            break;
        }
        let mut hint_input_id = None;
        if action.input() {
            if !declared.contains(action.id()) && planned.inputs.len() == MAX_INPUTS {
                // The button stays, but without a `hint-inputId` — pointing at
                // an id no `<input>` declares is rejected too.
                log::warn!(
                    "Toast action type declares more than {MAX_INPUTS} inputs; the text box \
                     for action {:?} is dropped",
                    action.id()
                );
            } else {
                if declared.insert(action.id()) {
                    planned.inputs.push(ToastInput {
                        id: action.id().to_string(),
                        placeholder: action.input_placeholder().map(ToString::to_string),
                    });
                } else {
                    log::debug!(
                        "Toast action {:?} reuses an input id already declared in this action \
                         type; emitting a single <input> for it",
                        action.id()
                    );
                }
                hint_input_id = Some(action.id().to_string());
            }
        }
        // For an input action the button is the submit button, so its caption
        // comes from `inputButtonTitle` ("Send") and not from `title`, which
        // labels the feature ("Reply"). Without the former we keep `title`.
        let content = if action.input() {
            action
                .input_button_title()
                .unwrap_or_else(|| action.title())
        } else {
            action.title()
        };
        planned.buttons.push(ToastButton {
            content: content.to_string(),
            arguments: encode_action_arguments(action.id(), launch),
            activation_type: if action.foreground() {
                "foreground"
            } else {
                "background"
            },
            hint_input_id,
        });
    }
    planned
}

/// Build the `arguments=` string for one toast button.
///
/// Windows hands a button activation that button's own `arguments=` and never
/// the toast's `launch=`, so a bare action id leaves a cold activation with no
/// way back to the notification it came from. Copying `launch` into every
/// button and adding `"action"` keeps the context and marks the payload as a
/// button activation in one object.
pub fn encode_action_arguments(action_id: &str, launch: &Value) -> String {
    let mut encoded = match launch {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    };
    encoded.insert(ACTION_KEY.to_string(), Value::String(action_id.to_string()));
    Value::Object(encoded).to_string()
}

/// Result of decoding a toast activation's `Arguments` string.
///
/// Shared by the in-process `Activated` handler and the out-of-proc COM
/// activator so the event shapes the JS layer sees are identical whichever
/// route a click took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedActivation {
    /// `"tap"` for the toast body, otherwise the activated action's id.
    pub action_id: String,
    /// Text typed into that action's box. `None` when the toast had no input,
    /// or the box was submitted empty.
    pub input_value: Option<String>,
    /// The `{"id": …, "data": …}` context recovered from `arguments=`, or
    /// `None` for a toast posted before this encoding existed.
    pub notification: Option<Value>,
    /// Whether this was an activation of the toast body rather than a button.
    pub is_tap: bool,
    /// Group the toast was posted under, when it had one.
    group: Option<String>,
}

impl DecodedActivation {
    /// `actionPerformed` payload.
    ///
    /// `notification` overrides the context recovered from `arguments=`: the
    /// in-process path still holds the full `ActiveNotification` it just
    /// posted, while the COM path only ever has the `{id, data}` the toast
    /// carried.
    pub fn action_payload(&self, notification: Option<Value>) -> Value {
        serde_json::json!({
            "actionId": self.action_id,
            "inputValue": self.input_value,
            "notification": notification
                .or_else(|| self.notification_fallback())
                .unwrap_or(Value::Null),
        })
    }

    /// The `notification` field of an `actionPerformed` payload, rebuilt from
    /// `arguments=` when the caller has nothing richer.
    ///
    /// The wire object is `{"id", "data"}`, but the warm route hands over a
    /// whole `ActiveNotification`, whose custom payload is named `extra`.
    /// Renaming here is what makes a cold activation and a warm one expose
    /// the caller's extras under the same key.
    fn notification_fallback(&self) -> Option<Value> {
        let Value::Object(wire) = self.notification.as_ref()? else {
            return None;
        };
        let mut rebuilt = Map::new();
        rebuilt.insert(
            "id".to_string(),
            wire.get("id").cloned().unwrap_or(Value::Null),
        );
        rebuilt.insert(
            "extra".to_string(),
            wire.get("data")
                .cloned()
                .unwrap_or_else(|| Value::Object(Map::new())),
        );
        Some(Value::Object(rebuilt))
    }

    /// The [`ClaimTable`] entry this activation belongs to, when the toast
    /// carried enough to identify itself.
    pub fn claim_key(&self) -> Option<ClaimKey> {
        Some((
            self.notification_id()?,
            self.group.clone().unwrap_or_default(),
        ))
    }

    /// Notification id recovered from `arguments=`, when the toast carried
    /// one. Used to decide which of the two activation routes delivers.
    pub fn notification_id(&self) -> Option<i32> {
        self.notification
            .as_ref()?
            .get("id")?
            .as_i64()
            .and_then(|id| i32::try_from(id).ok())
    }

    /// `notificationClicked` payload, or `None` for a button activation.
    pub fn click_payload(&self) -> Option<Value> {
        if !self.is_tap {
            return None;
        }
        Some(
            self.notification
                .clone()
                .unwrap_or_else(|| serde_json::json!({ "id": Value::Null, "data": {} })),
        )
    }
}

/// Decode an activation's `arguments=` plus the text boxes it submitted.
pub fn decode_activation(
    invoked_args: &str,
    inputs: &HashMap<String, String>,
) -> DecodedActivation {
    let parsed = serde_json::from_str::<Value>(invoked_args)
        .ok()
        .and_then(|value| match value {
            Value::Object(map) => Some(map),
            _ => None,
        });

    // Two independent axes: whether `invoked_args` decoded as one of the JSON
    // objects we write, and whether it is empty at all. Matched as a tuple so
    // the outcomes stay side by side.
    let (action_id, mut notification, is_tap) = match (parsed, invoked_args.is_empty()) {
        (Some(mut map), _) => match map.remove(ACTION_KEY) {
            // Button activation: `{"action": "<id>", "id": …, "data": …}`.
            // Removing `action` leaves exactly the `launch=` context.
            Some(Value::String(id)) => (id, Some(Value::Object(map)), false),
            // Body tap: the toast-level `launch=` object. An `action` key of
            // some other type isn't ours, so it is put back untouched.
            other => {
                if let Some(other) = other {
                    map.insert(ACTION_KEY.to_string(), other);
                }
                (TAP_ACTION_ID.to_string(), Some(Value::Object(map)), true)
            }
        },
        // Toast posted before `launch=` was set, or a tap carrying no extras.
        (None, true) => (TAP_ACTION_ID.to_string(), None, true),
        // Toast posted before this encoding existed: `arguments=` is the bare
        // action id.
        (None, false) => (invoked_args.to_string(), None, false),
    };

    // Lifted out of the context so the payloads stay `{id, data}`.
    let group = notification
        .as_mut()
        .and_then(Value::as_object_mut)
        .and_then(|map| map.remove(GROUP_KEY))
        .and_then(|value| value.as_str().map(ToString::to_string));

    let input_value = pick_input_value(inputs, &action_id);
    DecodedActivation {
        action_id,
        input_value,
        notification,
        is_tap,
        group,
    }
}

/// Pick the text the user typed for `action_id`.
///
/// `plan_toast_actions` names each `<input>` after its action, so the box
/// belonging to the activated action is authoritative: when it is present but
/// empty the user submitted it untouched, which is nothing typed — not a
/// reason to reach for some other action's text.
///
/// Only when no box carries the action's name is a fallback used, and only
/// for a lone entry, which can only be the box that was submitted. That
/// covers a toast posted by a build that named its inputs differently. With
/// several unnamed boxes there is no way to tell which belongs to this
/// action, so nothing is reported.
fn pick_input_value(inputs: &HashMap<String, String>, action_id: &str) -> Option<String> {
    if let Some(exact) = inputs.get(action_id) {
        return (!exact.is_empty()).then(|| exact.clone());
    }
    match inputs.iter().next() {
        Some((_, text)) if inputs.len() == 1 && !text.is_empty() => Some(text.clone()),
        _ => None,
    }
}

/// `HRESULT_FROM_WIN32(ERROR_NOT_FOUND)` — what `ToastNotifier::Setting()`
/// returns for an AUMID Windows holds no notification settings for.
pub const ERROR_NOT_FOUND_HRESULT: i32 = 0x8007_0490_u32.cast_signed();

/// Map a failed `ToastNotifier::Setting()` call onto a permission state, or
/// `None` to let the error through to the caller.
///
/// Windows creates `HKCU\…\Notifications\Settings\<AUMID>` lazily, when an
/// AUMID first shows a toast. Until then `Setting()` fails with
/// `ERROR_NOT_FOUND` for an unpackaged app, which means "no entry yet", not
/// "denied".
///
/// The answer is `Granted`, not `Prompt`. Windows has no runtime notification
/// prompt, so Tauri's `requestPermission` here only re-queries
/// `permission_state`. `Prompt` therefore wedges the canonical
/// `isPermissionGranted()` / `requestPermission()` loop forever: the app never
/// gets a `granted`, never sends the first toast, and the first toast is
/// exactly what would have created the settings entry. `Granted` lets the
/// send go ahead; if notifications really are off, `Show()` surfaces that at
/// the point it happens.
///
/// A packaged app always has the entry, so the same HRESULT there means the
/// AUMID is wrong (see `is_packaged` in `windows.rs`) and stays an error, as
/// does every other HRESULT on either flavor.
pub const fn permission_state_for_setting_error(
    hresult: i32,
    packaged: bool,
) -> Option<PermissionState> {
    if !packaged && hresult == ERROR_NOT_FOUND_HRESULT {
        Some(PermissionState::Granted)
    } else {
        None
    }
}

/// Upper bound on tracked toasts. Entries are tiny and only `Delivered` ones
/// are ever evicted, so a busy app loses nothing it still needs.
const MAX_CLAIMS: usize = 256;

/// Identifies one toast: its notification id plus its group, because ids are
/// caller-chosen and two groups may legitimately use the same one.
pub type ClaimKey = (i32, String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Claim {
    /// An in-process handler is live for this toast; the COM activator defers
    /// to it. Carries the generation of the `Show` that registered it.
    InProcess(u64),
    /// Already delivered. Keeps suppressing the other route until the next
    /// `Show` of the same key replaces it, or until it is evicted as one of the
    /// oldest settled entries once more than `MAX_CLAIMS` have accumulated
    /// (a late callback for such an old toast is then delivered again rather
    /// than lost; the bound trades that corner for bounded memory).
    Delivered(u64),
}

/// Which of the two activation routes may deliver a given toast's click.
///
/// When the COM activator is registered, both it and the in-process
/// `Activated` handler fire for the same click, on different threads and in
/// no guaranteed order, so one has to stand down. The in-process handler is
/// preferred: it still holds the whole `ActiveNotification` it posted, where
/// the COM path only has what the toast carried. Every decision is one
/// transition on this table under a single lock, so exactly one route
/// delivers whichever arrives first.
///
/// State is keyed to a monotonic generation rather than a clock: a toast's
/// entry survives until the next `Show` of the same key replaces it (or it
/// is evicted past `MAX_CLAIMS`, see `Claim::Delivered`), so a
/// late callback is suppressed deterministically however long it took, and a
/// re-show always starts fresh. Nothing here expires on its own.
#[derive(Debug, Default)]
pub struct ClaimTable {
    claims: HashMap<ClaimKey, Claim>,
    next_generation: u64,
}

impl ClaimTable {
    /// Record that `key`'s toast has a live in-process `Activated` handler,
    /// replacing whatever an earlier `Show` of the same key left behind.
    /// Returns the generation, which the dismissal handlers quote back so a
    /// callback from an earlier show cannot clear a newer entry.
    pub fn register_in_process(&mut self, key: ClaimKey) -> u64 {
        let generation = self.bump();
        self.claims.insert(key, Claim::InProcess(generation));
        self.evict();
        generation
    }

    /// Forget a live in-process handler without delivering: the toast was
    /// cancelled by the user or failed, so no activation is coming. Ignores a
    /// tombstone (that records a delivery) and an entry from a later `Show`.
    pub fn forget_in_process(&mut self, key: &ClaimKey, generation: u64) {
        if self.claims.get(key) == Some(&Claim::InProcess(generation)) {
            self.claims.remove(key);
        }
    }

    /// Deliver through the in-process handler registered by generation
    /// `generation`, if it is still the current one.
    ///
    /// A handler whose generation no longer matches belongs to an earlier
    /// `Show` of the same key: it must not consume the newer entry, and it
    /// must not dispatch, because its captured `ActiveNotification` describes
    /// a toast that has since been replaced.
    pub fn claim_in_process(&mut self, key: &ClaimKey, generation: u64) -> bool {
        if self.claims.get(key) != Some(&Claim::InProcess(generation)) {
            return false;
        }
        self.claims
            .insert(key.clone(), Claim::Delivered(generation));
        self.evict();
        true
    }

    /// Deliver through the COM activator, unless this process showed the
    /// toast itself and its own handler has it.
    ///
    /// Records nothing. The `Delivered` marker exists to silence a late COM
    /// callback after the in-process handler delivered, and a toast this
    /// process never showed has no in-process handler —
    /// so a scheduled toast, which is activated without ever registering,
    /// keeps delivering however many times it is clicked.
    pub fn claim_from_com(&self, key: &ClaimKey) -> bool {
        !self.claims.contains_key(key)
    }

    const fn bump(&mut self) -> u64 {
        let generation = self.next_generation;
        self.next_generation = self.next_generation.wrapping_add(1);
        generation
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.claims.len()
    }

    #[cfg(test)]
    fn delivered_len(&self) -> usize {
        self.claims
            .values()
            .filter(|claim| matches!(claim, Claim::Delivered(_)))
            .count()
    }

    /// Keep the table bounded by dropping the longest-settled deliveries.
    /// A live `InProcess` entry is never evicted: losing it would hand the
    /// click to the COM route with the poorer payload.
    fn evict(&mut self) {
        while self.claims.len() > MAX_CLAIMS {
            let oldest = self
                .claims
                .iter()
                .filter_map(|(key, claim)| match claim {
                    Claim::Delivered(generation) => Some((*generation, key.clone())),
                    Claim::InProcess(_) => None,
                })
                .min_by_key(|(generation, _)| *generation);
            let Some((_, key)) = oldest else {
                break;
            };
            self.claims.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(value: serde_json::Value) -> Action {
        serde_json::from_value(value).expect("action fixture should deserialize")
    }

    fn launch() -> Value {
        serde_json::json!({ "id": 42, "data": { "sessionId": "s-1" } })
    }

    fn inputs(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    // ==================== <actions> planning (#347) ====================

    #[test]
    fn plain_action_gets_no_input() {
        let planned = plan_toast_actions(
            &[action(
                serde_json::json!({ "id": "mark-read", "title": "Mark as Read" }),
            )],
            &launch(),
        );

        assert_eq!(planned.inputs, [] as [ToastInput; 0]);
        assert_eq!(planned.buttons.len(), 1);
        assert_eq!(planned.buttons[0].content, "Mark as Read");
        assert_eq!(planned.buttons[0].hint_input_id, None);
        assert_eq!(planned.buttons[0].activation_type, "background");
    }

    #[test]
    fn input_action_declares_box_and_wires_button_to_it() {
        let planned = plan_toast_actions(
            &[action(serde_json::json!({
                "id": "reply",
                "title": "Reply",
                "input": true,
                "inputPlaceholder": "Type your reply...",
                "inputButtonTitle": "Send",
                "foreground": true,
            }))],
            &launch(),
        );

        assert_eq!(
            planned.inputs,
            vec![ToastInput {
                id: "reply".to_string(),
                placeholder: Some("Type your reply...".to_string()),
            }]
        );
        // The submit button is captioned "Send", not "Reply", and points at
        // the box declared above it.
        assert_eq!(planned.buttons[0].content, "Send");
        assert_eq!(planned.buttons[0].hint_input_id, Some("reply".to_string()));
        assert_eq!(planned.buttons[0].activation_type, "foreground");
    }

    #[test]
    fn input_action_without_button_title_keeps_title() {
        let planned = plan_toast_actions(
            &[action(serde_json::json!({
                "id": "reply",
                "title": "Reply",
                "input": true,
            }))],
            &launch(),
        );

        assert_eq!(planned.inputs[0].placeholder, None);
        assert_eq!(planned.buttons[0].content, "Reply");
    }

    #[test]
    fn inputs_are_planned_ahead_of_every_button() {
        let planned = plan_toast_actions(
            &[
                action(serde_json::json!({ "id": "mark-read", "title": "Mark as Read" })),
                action(serde_json::json!({ "id": "reply", "title": "Reply", "input": true })),
            ],
            &launch(),
        );

        // `hint-inputId` may only name an already-declared id, so the single
        // input must be emitted before both buttons even though its action is
        // second in the list.
        assert_eq!(planned.inputs.len(), 1);
        assert_eq!(planned.inputs[0].id, "reply");
        assert_eq!(planned.buttons.len(), 2);
    }

    #[test]
    fn a_repeated_input_id_is_declared_once() {
        let planned = plan_toast_actions(
            &[
                action(serde_json::json!({ "id": "reply", "title": "Reply", "input": true })),
                action(serde_json::json!({ "id": "reply", "title": "Reply all", "input": true })),
            ],
            &launch(),
        );

        // Windows rejects a toast with two `<input>` elements sharing an id;
        // both buttons point at the single box instead.
        assert_eq!(planned.inputs.len(), 1);
        assert_eq!(planned.inputs[0].id, "reply");
        assert_eq!(planned.buttons.len(), 2);
        assert!(
            planned
                .buttons
                .iter()
                .all(|b| b.hint_input_id.as_deref() == Some("reply"))
        );
    }

    // ==================== arguments round trip (#350) ====================

    #[test]
    fn button_arguments_carry_the_notification_context() {
        let encoded = encode_action_arguments("reply", &launch());
        let decoded = decode_activation(&encoded, &HashMap::new());

        assert_eq!(decoded.action_id, "reply");
        assert!(!decoded.is_tap);
        assert_eq!(decoded.click_payload(), None);
        assert_eq!(
            decoded.action_payload(None),
            serde_json::json!({
                "actionId": "reply",
                "inputValue": Value::Null,
                "notification": { "id": 42, "extra": { "sessionId": "s-1" } },
            })
        );
    }

    #[test]
    fn toast_level_launch_still_decodes_as_a_tap() {
        let decoded = decode_activation(&launch().to_string(), &HashMap::new());

        assert_eq!(decoded.action_id, "tap");
        assert!(decoded.is_tap);
        assert_eq!(decoded.click_payload(), Some(launch()));
    }

    #[test]
    fn empty_arguments_decode_as_a_tap_with_no_context() {
        let decoded = decode_activation("", &HashMap::new());

        assert_eq!(decoded.action_id, "tap");
        assert!(decoded.is_tap);
        assert_eq!(decoded.notification, None);
        assert_eq!(
            decoded.click_payload(),
            Some(serde_json::json!({ "id": Value::Null, "data": {} }))
        );
    }

    #[test]
    fn legacy_bare_action_id_still_decodes() {
        let decoded = decode_activation("reply", &HashMap::new());

        assert_eq!(decoded.action_id, "reply");
        assert!(!decoded.is_tap);
        assert_eq!(decoded.notification, None);
        assert_eq!(
            decoded.action_payload(None),
            serde_json::json!({
                "actionId": "reply",
                "inputValue": Value::Null,
                "notification": Value::Null,
            })
        );
    }

    #[test]
    fn a_foreign_action_key_is_not_mistaken_for_a_button() {
        // Someone else's `launch=` payload that happens to carry `action`.
        let decoded = decode_activation(r#"{"action":{"kind":"open"},"id":7}"#, &HashMap::new());

        assert_eq!(decoded.action_id, "tap");
        assert!(decoded.is_tap);
        assert_eq!(
            decoded.notification,
            Some(serde_json::json!({ "action": { "kind": "open" }, "id": 7 }))
        );
    }

    #[test]
    fn non_object_json_arguments_are_treated_as_an_action_id() {
        // `"42"` parses as JSON but isn't an object, so it is an action id.
        let decoded = decode_activation("42", &HashMap::new());
        assert_eq!(decoded.action_id, "42");
        assert!(!decoded.is_tap);
    }

    #[test]
    fn action_payload_prefers_the_caller_supplied_notification() {
        let encoded = encode_action_arguments("reply", &launch());
        let decoded = decode_activation(&encoded, &HashMap::new());
        let richer = serde_json::json!({ "id": 42, "title": "New Message" });

        assert_eq!(
            decoded.action_payload(Some(richer.clone()))["notification"],
            richer
        );
    }

    #[test]
    fn the_notification_id_is_recoverable_from_either_shape() {
        let button = decode_activation(
            &encode_action_arguments("reply", &launch()),
            &HashMap::new(),
        );
        let tap = decode_activation(&launch().to_string(), &HashMap::new());

        assert_eq!(button.notification_id(), Some(42));
        assert_eq!(tap.notification_id(), Some(42));
        // A toast posted before `launch=` existed has nothing to key on.
        assert_eq!(
            decode_activation("reply", &HashMap::new()).notification_id(),
            None
        );
        assert_eq!(
            decode_activation("", &HashMap::new()).notification_id(),
            None
        );
    }

    #[test]
    fn both_routes_expose_the_extras_under_the_same_key() {
        let decoded = decode_activation(
            &encode_action_arguments("reply", &launch()),
            &HashMap::new(),
        );

        // Cold: rebuilt from `arguments=`.
        let cold = decoded.action_payload(None);
        // Warm: the whole `ActiveNotification`, whose field is named `extra`.
        let warm = decoded.action_payload(Some(serde_json::json!({
            "id": 42, "title": "New Message", "extra": { "sessionId": "s-1" },
        })));

        assert_eq!(cold["notification"]["extra"]["sessionId"], "s-1");
        assert_eq!(warm["notification"]["extra"]["sessionId"], "s-1");
        assert!(cold["notification"].get("data").is_none());
    }

    #[test]
    fn a_sixth_action_and_a_sixth_input_are_dropped() {
        let many: Vec<Action> = (0..7)
            .map(|i| {
                action(serde_json::json!({
                    "id": format!("a{i}"), "title": format!("A{i}"), "input": true,
                }))
            })
            .collect();
        let planned = plan_toast_actions(&many, &launch());

        assert_eq!(planned.buttons.len(), 5);
        assert_eq!(planned.inputs.len(), 5);
        // Every surviving button points at a box that exists.
        for button in &planned.buttons {
            let id = button.hint_input_id.as_deref().expect("input action");
            assert!(planned.inputs.iter().any(|i| i.id == id));
        }
    }

    #[test]
    fn the_group_rides_along_without_reaching_either_payload() {
        let launch = serde_json::json!({ "id": 42, "data": {}, "group": "chat" });
        let decoded =
            decode_activation(&encode_action_arguments("reply", &launch), &HashMap::new());

        assert_eq!(decoded.claim_key(), Some((42, "chat".to_string())));
        // Neither payload gains a `group` field.
        assert_eq!(
            decoded.action_payload(None)["notification"],
            serde_json::json!({ "id": 42, "extra": {} })
        );
        let tap = decode_activation(&launch.to_string(), &HashMap::new());
        assert_eq!(
            tap.click_payload(),
            Some(serde_json::json!({ "id": 42, "data": {} }))
        );
    }

    #[test]
    fn an_ungrouped_toast_claims_under_the_empty_group() {
        let decoded = decode_activation(
            &encode_action_arguments("reply", &launch()),
            &HashMap::new(),
        );
        assert_eq!(decoded.claim_key(), Some((42, String::new())));
        assert_eq!(
            decode_activation("reply", &HashMap::new()).claim_key(),
            None
        );
    }

    // ==================== activation claims (#351) ====================

    fn key(id: i32) -> ClaimKey {
        (id, String::new())
    }

    #[test]
    fn the_warm_route_delivers_when_it_arrives_first() {
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));

        assert!(table.claim_in_process(&key(1), generation));
        assert!(!table.claim_from_com(&key(1)));
    }

    #[test]
    fn the_com_route_defers_when_it_arrives_first() {
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));

        // COM stands down without consuming the claim.
        assert!(!table.claim_from_com(&key(1)));
        assert!(table.claim_in_process(&key(1), generation));
    }

    #[test]
    fn a_cold_activation_delivers_and_records_nothing() {
        let table = ClaimTable::default();

        assert!(table.claim_from_com(&key(1)));
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn repeated_cold_activations_of_the_same_key_both_deliver() {
        // A scheduled toast is activated without this process ever having
        // registered a handler for it, and may be clicked more than once.
        let table = ClaimTable::default();

        assert!(table.claim_from_com(&key(1)));
        assert!(table.claim_from_com(&key(1)));
    }

    #[test]
    fn a_late_com_callback_after_a_warm_delivery_is_suppressed() {
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));
        assert!(table.claim_in_process(&key(1), generation));

        // However long it took, and however much else happened in between.
        for other in 2..50 {
            table.register_in_process(key(other));
        }
        assert!(!table.claim_from_com(&key(1)));
    }

    #[test]
    fn a_stale_warm_handler_cannot_consume_a_newer_show() {
        let mut table = ClaimTable::default();
        let stale = table.register_in_process(key(1));
        let current = table.register_in_process(key(1));

        // The first show's handler fires late: it describes a toast that has
        // been replaced, so it neither dispatches nor spends the new claim.
        assert!(!table.claim_in_process(&key(1), stale));
        assert!(table.claim_in_process(&key(1), current));
    }

    #[test]
    fn a_reshow_of_a_delivered_id_starts_fresh() {
        let mut table = ClaimTable::default();
        let first = table.register_in_process(key(1));
        assert!(table.claim_in_process(&key(1), first));

        let second = table.register_in_process(key(1));
        assert!(table.claim_in_process(&key(1), second));
    }

    #[test]
    fn two_toasts_with_different_ids_both_deliver() {
        let mut table = ClaimTable::default();
        let a = table.register_in_process(key(1));
        let b = table.register_in_process(key(2));

        assert!(table.claim_in_process(&key(1), a));
        assert!(table.claim_in_process(&key(2), b));
    }

    #[test]
    fn the_same_id_in_two_groups_does_not_collide() {
        let mut table = ClaimTable::default();
        let chat = table.register_in_process((1, "chat".to_string()));
        let mail = table.register_in_process((1, "mail".to_string()));

        assert!(table.claim_in_process(&(1, "chat".to_string()), chat));
        assert!(table.claim_in_process(&(1, "mail".to_string()), mail));
    }

    #[test]
    fn a_dismissal_releases_the_claim_but_not_a_tombstone() {
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));
        table.forget_in_process(&key(1), generation);
        // No live handler: COM is free to deliver.
        assert!(table.claim_from_com(&key(1)));

        // After a delivery, forgetting must not resurrect the activation.
        let generation = table.register_in_process(key(1));
        assert!(table.claim_in_process(&key(1), generation));
        table.forget_in_process(&key(1), generation);
        assert!(!table.claim_from_com(&key(1)));
    }

    #[test]
    fn a_stale_dismissal_cannot_clear_a_newer_show() {
        let mut table = ClaimTable::default();
        let stale = table.register_in_process(key(1));
        let current = table.register_in_process(key(1));

        // The first show's Dismissed callback arrives after the re-show.
        table.forget_in_process(&key(1), stale);
        assert!(!table.claim_from_com(&key(1)));
        assert!(table.claim_in_process(&key(1), current));
    }

    #[test]
    fn a_rolled_back_registration_leaves_no_entry() {
        // `show()` releases the claim when attaching or `Show()` fails, so a
        // toast that never appeared cannot silence the COM route later.
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));
        table.forget_in_process(&key(1), generation);

        assert_eq!(table.len(), 0);
        assert!(table.claim_from_com(&key(1)));
    }

    #[test]
    fn a_timed_out_toast_keeps_preferring_the_warm_route() {
        // `windows.rs` does not call `forget_in_process` for `TimedOut`: the
        // toast moved to Action Center, and a click from there should still
        // reach the richer in-process payload while the process is alive.
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));

        assert!(!table.claim_from_com(&key(1)));
        assert!(table.claim_in_process(&key(1), generation));
    }

    #[test]
    fn settled_entries_stay_bounded() {
        let mut table = ClaimTable::default();
        let generations: Vec<(i32, u64)> = (0..300)
            .map(|id| (id, table.register_in_process(key(id))))
            .collect();

        // Live handlers are never evicted, however many there are.
        assert_eq!(table.len(), 300);

        for (id, generation) in generations {
            assert!(table.claim_in_process(&key(id), generation));
        }
        assert!(
            table.delivered_len() <= 256,
            "settled entries: {}",
            table.delivered_len()
        );
    }

    // ==================== typed input (#348) ====================

    #[test]
    fn typed_text_reaches_the_payload() {
        let encoded = encode_action_arguments("reply", &launch());
        let decoded = decode_activation(&encoded, &inputs(&[("reply", "on my way")]));

        assert_eq!(decoded.input_value, Some("on my way".to_string()));
        assert_eq!(
            decoded.action_payload(None)["inputValue"],
            serde_json::json!("on my way")
        );
    }

    #[test]
    fn the_box_belonging_to_the_activated_action_wins() {
        let encoded = encode_action_arguments("reply", &launch());
        let decoded = decode_activation(
            &encoded,
            &inputs(&[("aaa-other", "wrong"), ("reply", "right")]),
        );

        assert_eq!(decoded.input_value, Some("right".to_string()));
    }

    #[test]
    fn a_lone_mismatched_box_is_still_delivered() {
        // Toast posted by an older build: the input id doesn't match the
        // action id, but a single box can only be the one submitted.
        let decoded = decode_activation("reply", &inputs(&[("input-reply", "earlier")]));

        assert_eq!(decoded.input_value, Some("earlier".to_string()));
    }

    #[test]
    fn several_mismatched_boxes_are_ambiguous_and_report_nothing() {
        let decoded = decode_activation(
            "reply",
            &inputs(&[("zzz", "later"), ("input-reply", "earlier")]),
        );

        assert_eq!(decoded.input_value, None);
    }

    #[test]
    fn an_untouched_box_reports_nothing_typed() {
        let decoded = decode_activation("reply", &inputs(&[("reply", "")]));
        assert_eq!(decoded.input_value, None);
        assert_eq!(
            decoded.action_payload(None)["inputValue"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn an_empty_own_box_does_not_borrow_another_actions_text() {
        // README promises `inputValue` is null when the box was left empty;
        // the neighbouring box's text is not this action's input.
        let decoded = decode_activation("reply", &inputs(&[("reply", ""), ("note", "typed")]));

        assert_eq!(decoded.input_value, None);
    }

    // ==================== permission mapping (#355) ====================

    #[test]
    fn unpackaged_error_not_found_lets_the_first_toast_through() {
        // `Prompt` would wedge the isPermissionGranted/requestPermission loop:
        // Windows has no runtime prompt, so the app would never reach the
        // first toast — the thing that creates the settings entry.
        assert_eq!(
            permission_state_for_setting_error(ERROR_NOT_FOUND_HRESULT, false),
            Some(PermissionState::Granted)
        );
    }

    #[test]
    fn packaged_error_not_found_stays_an_error() {
        assert_eq!(
            permission_state_for_setting_error(ERROR_NOT_FOUND_HRESULT, true),
            None
        );
    }

    #[test]
    fn other_hresults_stay_errors() {
        // E_ACCESSDENIED and RPC_E_CHANGED_MODE.
        for hresult in [0x8007_0005_u32.cast_signed(), 0x8001_010E_u32.cast_signed()] {
            assert_eq!(permission_state_for_setting_error(hresult, false), None);
            assert_eq!(permission_state_for_setting_error(hresult, true), None);
        }
    }
}
