//! Pure parts of the Windows toast backend, split out of `windows` — which
//! only compiles on Windows — so these rules are unit-tested on any host.

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};
use tauri::plugin::PermissionState;

use crate::models::Action;

const ACTION_KEY: &str = "action";
/// Rides in `arguments=` so a cold activation can rebuild the [`ClaimKey`].
const GROUP_KEY: &str = "group";
const TAP_ACTION_ID: &str = "tap";

/// Windows rejects an `<actions>` block holding more than five of either.
const MAX_INPUTS: usize = 5;
const MAX_BUTTONS: usize = 5;

#[derive(Debug, PartialEq, Eq)]
pub struct ToastInput {
    pub id: String,
    pub placeholder: Option<String>,
}

#[derive(Debug)]
pub struct ToastButton {
    pub content: String,
    pub arguments: String,
    pub activation_type: &'static str,
    pub hint_input_id: Option<String>,
}

/// Inputs are a separate list because a `hint-inputId` may only name an
/// `<input>` that already exists in the document.
#[derive(Debug, Default)]
pub struct ToastActions {
    pub inputs: Vec<ToastInput>,
    pub buttons: Vec<ToastButton>,
}

pub fn plan_toast_actions(actions: &[Action], launch: &Value) -> ToastActions {
    let mut planned = ToastActions::default();
    let mut declared: HashSet<&str> = HashSet::new();

    for action in actions {
        if planned.buttons.len() == MAX_BUTTONS {
            log::warn!(
                "Toast action type declares more than {MAX_BUTTONS} actions; dropping the rest"
            );
            break;
        }

        let mut hint_input_id = None;
        if action.input() {
            if declared.contains(action.id()) || planned.inputs.len() < MAX_INPUTS {
                if declared.insert(action.id()) {
                    planned.inputs.push(ToastInput {
                        id: action.id().to_string(),
                        placeholder: action.input_placeholder().map(ToString::to_string),
                    });
                }
                hint_input_id = Some(action.id().to_string());
            } else {
                // A `hint-inputId` naming a box that was dropped is rejected too.
                log::warn!(
                    "Toast action type declares more than {MAX_INPUTS} inputs; dropping the text \
                     box for action {:?}",
                    action.id()
                );
            }
        }

        // An input action's button submits the box, so `inputButtonTitle`
        // ("Send") captions it rather than `title` ("Reply").
        let content = if action.input() {
            action
                .input_button_title()
                .unwrap_or_else(|| action.title())
        } else {
            action.title()
        };

        // Windows hands a button activation only its own `arguments=`, never
        // the toast's `launch=`, so the context is copied into every button.
        let mut arguments = match launch {
            Value::Object(map) => map.clone(),
            _ => Map::new(),
        };
        arguments.insert(
            ACTION_KEY.to_string(),
            Value::String(action.id().to_string()),
        );

        planned.buttons.push(ToastButton {
            content: content.to_string(),
            arguments: Value::Object(arguments).to_string(),
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

#[derive(Debug)]
pub struct DecodedActivation {
    pub is_tap: bool,
    action_id: String,
    input_value: Option<String>,
    notification: Option<Value>,
    group: Option<String>,
}

impl DecodedActivation {
    /// `actionPerformed` payload. A caller-supplied `notification` wins over the
    /// one recovered from `arguments=`, whose `data` is renamed `extra` to match.
    pub fn action_payload(&self, notification: Option<Value>) -> Value {
        let notification = notification.or_else(|| {
            let wire = self.notification.as_ref()?.as_object()?;
            Some(serde_json::json!({
                "id": wire.get("id").cloned().unwrap_or(Value::Null),
                "extra": wire
                    .get("data")
                    .cloned()
                    .unwrap_or_else(|| Value::Object(Map::new())),
            }))
        });
        serde_json::json!({
            "actionId": self.action_id,
            "inputValue": self.input_value,
            "notification": notification.unwrap_or(Value::Null),
        })
    }

    /// `notificationClicked` payload, or `None` for a button activation.
    pub fn click_payload(&self) -> Option<Value> {
        self.is_tap.then(|| {
            self.notification
                .clone()
                .unwrap_or_else(|| serde_json::json!({ "id": Value::Null, "data": {} }))
        })
    }

    pub fn claim_key(&self) -> Option<ClaimKey> {
        let id = self.notification.as_ref()?.get("id")?.as_i64()?;
        Some((
            i32::try_from(id).ok()?,
            self.group.clone().unwrap_or_default(),
        ))
    }
}

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

    let (action_id, mut notification, is_tap) = match (parsed, invoked_args.is_empty()) {
        // Removing `action` leaves exactly what `launch=` carried.
        (Some(mut map), _) => match map.remove(ACTION_KEY) {
            Some(Value::String(id)) => (id, Some(Value::Object(map)), false),
            other => {
                if let Some(other) = other {
                    map.insert(ACTION_KEY.to_string(), other);
                }
                (TAP_ACTION_ID.to_string(), Some(Value::Object(map)), true)
            }
        },
        // Posted before `launch=` was set, or a tap carrying no extras.
        (None, true) => (TAP_ACTION_ID.to_string(), None, true),
        // Posted before this encoding existed: `arguments=` is the bare id.
        (None, false) => (invoked_args.to_string(), None, false),
    };

    let group = notification
        .as_mut()
        .and_then(Value::as_object_mut)
        .and_then(|map| map.remove(GROUP_KEY))
        .and_then(|value| value.as_str().map(ToString::to_string));
    let input_value = pick_input_value(inputs, &action_id);

    DecodedActivation {
        is_tap,
        action_id,
        input_value,
        notification,
        group,
    }
}

/// The box named after the activated action wins even when empty: an empty box
/// was submitted untouched, not a reason to report a neighbour's text. A lone
/// box under another name can only be the one submitted; several cannot.
fn pick_input_value(inputs: &HashMap<String, String>, action_id: &str) -> Option<String> {
    if let Some(exact) = inputs.get(action_id) {
        return (!exact.is_empty()).then(|| exact.clone());
    }
    match inputs.iter().next() {
        Some((_, text)) if inputs.len() == 1 && !text.is_empty() => Some(text.clone()),
        _ => None,
    }
}

/// `HRESULT_FROM_WIN32(ERROR_NOT_FOUND)`.
const ERROR_NOT_FOUND_HRESULT: i32 = 0x8007_0490_u32.cast_signed();

/// Map a failed `ToastNotifier::Setting()` onto a permission state; `None`
/// lets the error through. An unpackaged AUMID has no settings entry until it
/// has shown a toast, and `Setting()` fails with `ERROR_NOT_FOUND` until then
/// (observed, not documented). `Prompt` would wedge the caller — Windows has
/// no runtime prompt, so `requestPermission` only re-queries this — and the
/// first toast is what would create the entry. A packaged app always has one.
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

const MAX_CLAIMS: usize = 256;

/// A toast's id plus its group: a caller-chosen id can recur across groups.
pub type ClaimKey = (i32, String);

#[derive(Debug, PartialEq, Eq)]
enum Claim {
    InProcess(u64),
    Delivered(u64),
}

/// Which of the two activation routes may deliver a toast's click.
///
/// With the COM activator registered, both it and the in-process `Activated`
/// handler fire for the same click in no guaranteed order, so one stands down;
/// the in-process one wins because it holds the `ActiveNotification` it posted.
/// Keyed by a monotonic generation, not a clock, so a late callback is
/// suppressed however long it took and a re-show starts fresh.
#[derive(Debug, Default)]
pub struct ClaimTable {
    claims: HashMap<ClaimKey, Claim>,
    next_generation: u64,
}

impl ClaimTable {
    /// Returns the generation that dismissal and activation callbacks quote
    /// back, so one belonging to an earlier show cannot touch a newer entry.
    pub fn register_in_process(&mut self, key: ClaimKey) -> u64 {
        let generation = self.next_generation;
        self.next_generation = self.next_generation.wrapping_add(1);
        self.claims.insert(key, Claim::InProcess(generation));
        self.evict();
        generation
    }

    pub fn forget_in_process(&mut self, key: &ClaimKey, generation: u64) {
        if self.claims.get(key) == Some(&Claim::InProcess(generation)) {
            self.claims.remove(key);
        }
    }

    pub fn claim_in_process(&mut self, key: &ClaimKey, generation: u64) -> bool {
        if self.claims.get(key) != Some(&Claim::InProcess(generation)) {
            return false;
        }
        self.claims
            .insert(key.clone(), Claim::Delivered(generation));
        self.evict();
        true
    }

    /// Records nothing: `Delivered` only silences a late COM callback after an
    /// in-process delivery, and a toast this process never showed has none.
    pub fn claim_from_com(&self, key: &ClaimKey) -> bool {
        !self.claims.contains_key(key)
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.claims.len()
    }

    /// Drops the longest-settled deliveries. A live `InProcess` entry is never
    /// evicted: losing it would hand the click to the poorer payload.
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
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    fn key(id: i32) -> ClaimKey {
        (id, String::new())
    }

    #[test]
    fn an_input_action_declares_a_box_and_points_its_button_at_it() {
        let planned = plan_toast_actions(
            &[
                action(serde_json::json!({ "id": "mark-read", "title": "Mark as Read" })),
                action(serde_json::json!({
                    "id": "reply",
                    "title": "Reply",
                    "input": true,
                    "inputPlaceholder": "Type your reply...",
                    "inputButtonTitle": "Send",
                    "foreground": true,
                })),
            ],
            &launch(),
        );

        assert_eq!(
            planned.inputs,
            [ToastInput {
                id: "reply".to_string(),
                placeholder: Some("Type your reply...".to_string()),
            }]
        );
        assert_eq!(planned.buttons[0].content, "Mark as Read");
        assert_eq!(planned.buttons[0].hint_input_id, None);
        assert_eq!(planned.buttons[0].activation_type, "background");
        assert_eq!(planned.buttons[1].content, "Send");
        assert_eq!(planned.buttons[1].hint_input_id, Some("reply".to_string()));
        assert_eq!(planned.buttons[1].activation_type, "foreground");
    }

    #[test]
    fn a_sixth_action_is_dropped_before_windows_rejects_the_whole_toast() {
        let many: Vec<Action> = (0..7)
            .map(|i| {
                action(serde_json::json!({
                    "id": format!("a{i}"), "title": format!("A{i}"), "input": true,
                }))
            })
            .collect();

        let planned = plan_toast_actions(&many, &launch());

        assert_eq!(planned.buttons.len(), MAX_BUTTONS);
        assert_eq!(planned.inputs.len(), MAX_INPUTS);
        for button in &planned.buttons {
            let id = button.hint_input_id.as_deref().expect("input action");
            assert!(planned.inputs.iter().any(|input| input.id == id));
        }
    }

    #[test]
    fn a_button_activation_round_trips_the_launch_context() {
        let planned = plan_toast_actions(
            &[action(
                serde_json::json!({ "id": "reply", "title": "Reply" }),
            )],
            &launch(),
        );

        let decoded = decode_activation(&planned.buttons[0].arguments, &HashMap::new());

        assert!(!decoded.is_tap);
        assert_eq!(decoded.click_payload(), None);
        assert_eq!(decoded.claim_key(), Some((42, String::new())));
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
    fn a_toast_level_launch_decodes_as_a_tap() {
        let decoded = decode_activation(&launch().to_string(), &HashMap::new());

        assert!(decoded.is_tap);
        assert_eq!(decoded.action_id, "tap");
        assert_eq!(decoded.click_payload(), Some(launch()));
    }

    #[test]
    fn arguments_written_before_this_encoding_still_decode() {
        let bare = decode_activation("reply", &HashMap::new());
        assert_eq!(bare.action_id, "reply");
        assert!(!bare.is_tap);
        assert_eq!(bare.claim_key(), None);

        let empty = decode_activation("", &HashMap::new());
        assert!(empty.is_tap);
        assert_eq!(
            empty.click_payload(),
            Some(serde_json::json!({ "id": Value::Null, "data": {} }))
        );
    }

    #[test]
    fn the_group_rides_in_the_arguments_without_reaching_the_payloads() {
        let launch = serde_json::json!({ "id": 42, "data": {}, "group": "chat" });
        let planned = plan_toast_actions(
            &[action(
                serde_json::json!({ "id": "reply", "title": "Reply" }),
            )],
            &launch,
        );

        let button = decode_activation(&planned.buttons[0].arguments, &HashMap::new());
        assert_eq!(button.claim_key(), Some((42, "chat".to_string())));
        assert_eq!(
            button.action_payload(None)["notification"],
            serde_json::json!({ "id": 42, "extra": {} })
        );

        let tap = decode_activation(&launch.to_string(), &HashMap::new());
        assert_eq!(
            tap.click_payload(),
            Some(serde_json::json!({ "id": 42, "data": {} }))
        );
    }

    #[test]
    fn the_box_named_after_the_activated_action_wins_even_when_empty() {
        let typed = decode_activation("reply", &inputs(&[("note", "wrong"), ("reply", "right")]));
        assert_eq!(typed.action_payload(None)["inputValue"], "right");

        let untouched = decode_activation("reply", &inputs(&[("reply", ""), ("note", "typed")]));
        assert_eq!(untouched.input_value, None);
    }

    #[test]
    fn only_a_lone_unmatched_box_is_used_as_a_fallback() {
        let lone = decode_activation("reply", &inputs(&[("input-reply", "earlier")]));
        assert_eq!(lone.input_value, Some("earlier".to_string()));

        let ambiguous = decode_activation(
            "reply",
            &inputs(&[("input-reply", "earlier"), ("zzz", "later")]),
        );
        assert_eq!(ambiguous.input_value, None);
    }

    #[test]
    fn a_late_com_callback_after_an_in_process_delivery_is_suppressed() {
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));

        assert!(table.claim_in_process(&key(1), generation));
        assert!(!table.claim_from_com(&key(1)));
    }

    #[test]
    fn the_com_route_stands_down_while_an_in_process_handler_holds_the_toast() {
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));

        assert!(!table.claim_from_com(&key(1)));
        assert!(table.claim_in_process(&key(1), generation));
    }

    #[test]
    fn a_dismissal_releases_the_claim_but_cannot_undo_a_delivery() {
        let mut table = ClaimTable::default();
        let generation = table.register_in_process(key(1));
        table.forget_in_process(&key(1), generation);
        assert!(table.claim_from_com(&key(1)));

        let generation = table.register_in_process(key(1));
        assert!(table.claim_in_process(&key(1), generation));
        table.forget_in_process(&key(1), generation);
        assert!(!table.claim_from_com(&key(1)));
    }

    #[test]
    fn a_callback_from_an_earlier_show_cannot_touch_the_current_claim() {
        let mut table = ClaimTable::default();
        let stale = table.register_in_process(key(1));
        let current = table.register_in_process(key(1));

        table.forget_in_process(&key(1), stale);
        assert!(!table.claim_in_process(&key(1), stale));
        assert!(table.claim_in_process(&key(1), current));
    }

    #[test]
    fn eviction_past_the_cap_keeps_the_newest_deliveries() {
        let mut table = ClaimTable::default();
        let newest = 300;
        for id in 0..=newest {
            let generation = table.register_in_process(key(id));
            assert!(table.claim_in_process(&key(id), generation));
        }

        assert_eq!(table.len(), MAX_CLAIMS);
        assert!(!table.claim_from_com(&key(newest)));
        assert!(table.claim_from_com(&key(0)));
    }

    #[test]
    fn only_an_unpackaged_missing_settings_entry_counts_as_granted() {
        assert_eq!(
            permission_state_for_setting_error(ERROR_NOT_FOUND_HRESULT, false),
            Some(PermissionState::Granted)
        );
        assert_eq!(
            permission_state_for_setting_error(ERROR_NOT_FOUND_HRESULT, true),
            None
        );
        // E_ACCESSDENIED
        assert_eq!(
            permission_state_for_setting_error(0x8007_0005_u32.cast_signed(), false),
            None
        );
    }
}
