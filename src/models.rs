// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fmt::Display};

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use tauri::plugin::PermissionState;

use url::Url;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionResponse {
    pub permission_state: PermissionState,
}

#[cfg(feature = "push-notifications")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationResponse {
    pub device_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    id: String,
    url: Url,
}

impl Attachment {
    pub fn new(id: impl Into<String>, url: Url) -> Self {
        Self { id: id.into(), url }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleInterval {
    pub year: Option<u8>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub weekday: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
}

#[derive(Debug)]
pub enum ScheduleEvery {
    Year,
    Month,
    TwoWeeks,
    Week,
    Day,
    Hour,
    Minute,
    Second,
}

impl Display for ScheduleEvery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Year => "year",
                Self::Month => "month",
                Self::TwoWeeks => "twoWeeks",
                Self::Week => "week",
                Self::Day => "day",
                Self::Hour => "hour",
                Self::Minute => "minute",
                Self::Second => "second",
            }
        )
    }
}

impl Serialize for ScheduleEvery {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl<'de> Deserialize<'de> for ScheduleEvery {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "year" => Ok(Self::Year),
            "month" => Ok(Self::Month),
            "twoweeks" => Ok(Self::TwoWeeks),
            "week" => Ok(Self::Week),
            "day" => Ok(Self::Day),
            "hour" => Ok(Self::Hour),
            "minute" => Ok(Self::Minute),
            "second" => Ok(Self::Second),
            _ => Err(DeError::custom(format!("unknown every kind '{s}'"))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Schedule {
    #[serde(rename_all = "camelCase")]
    At {
        #[serde(
            serialize_with = "iso8601::serialize",
            deserialize_with = "time::serde::iso8601::deserialize"
        )]
        date: time::OffsetDateTime,
        #[serde(default)]
        repeating: bool,
        #[serde(default)]
        allow_while_idle: bool,
    },
    #[serde(rename_all = "camelCase")]
    Interval {
        interval: ScheduleInterval,
        #[serde(default)]
        allow_while_idle: bool,
    },
    #[serde(rename_all = "camelCase")]
    Every {
        interval: ScheduleEvery,
        count: u8,
        #[serde(default)]
        allow_while_idle: bool,
    },
}

// custom ISO-8601 serialization that does not use 6 digits for years.
mod iso8601 {
    use serde::{ser::Error as _, Serialize, Serializer};
    use time::{
        format_description::well_known::iso8601::{Config, EncodedConfig},
        format_description::well_known::Iso8601,
        OffsetDateTime,
    };

    const SERDE_CONFIG: EncodedConfig = Config::DEFAULT.encode();

    pub fn serialize<S: Serializer>(
        datetime: &OffsetDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        datetime
            .format(&Iso8601::<SERDE_CONFIG>)
            .map_err(S::Error::custom)?
            .serialize(serializer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationData {
    #[serde(default = "default_id")]
    pub(crate) id: i32,
    pub(crate) channel_id: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) schedule: Option<Schedule>,
    pub(crate) large_body: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) action_type_id: Option<String>,
    pub(crate) group: Option<String>,
    #[serde(default)]
    pub(crate) group_summary: bool,
    pub(crate) sound: Option<String>,
    #[serde(default)]
    pub(crate) inbox_lines: Vec<String>,
    pub(crate) icon: Option<String>,
    pub(crate) large_icon: Option<String>,
    pub(crate) icon_color: Option<String>,
    #[serde(default)]
    pub(crate) attachments: Vec<Attachment>,
    #[serde(default)]
    pub(crate) extra: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) ongoing: bool,
    #[serde(default)]
    pub(crate) auto_cancel: bool,
    #[serde(default)]
    pub(crate) silent: bool,
}

fn default_id() -> i32 {
    rand::random()
}

impl Default for NotificationData {
    fn default() -> Self {
        Self {
            id: default_id(),
            channel_id: None,
            title: None,
            body: None,
            schedule: None,
            large_body: None,
            summary: None,
            action_type_id: None,
            group: None,
            group_summary: false,
            sound: None,
            inbox_lines: Vec::new(),
            icon: None,
            large_icon: None,
            icon_color: None,
            attachments: Vec::new(),
            extra: Default::default(),
            ongoing: false,
            auto_cancel: false,
            silent: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingNotification {
    id: i32,
    title: Option<String>,
    body: Option<String>,
    schedule: Schedule,
}

impl PendingNotification {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveNotification {
    id: i32,
    tag: Option<String>,
    title: Option<String>,
    body: Option<String>,
    group: Option<String>,
    #[serde(default)]
    group_summary: bool,
    #[serde(default)]
    data: HashMap<String, String>,
    #[serde(default)]
    extra: HashMap<String, serde_json::Value>,
    #[serde(default)]
    attachments: Vec<Attachment>,
    action_type_id: Option<String>,
    schedule: Option<Schedule>,
    sound: Option<String>,
}

impl ActiveNotification {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    pub fn group_summary(&self) -> bool {
        self.group_summary
    }

    pub fn data(&self) -> &HashMap<String, String> {
        &self.data
    }

    pub fn extra(&self) -> &HashMap<String, serde_json::Value> {
        &self.extra
    }

    pub fn attachments(&self) -> &[Attachment] {
        &self.attachments
    }

    pub fn action_type_id(&self) -> Option<&str> {
        self.action_type_id.as_deref()
    }

    pub fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    pub fn sound(&self) -> Option<&str> {
        self.sound.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionType {
    id: String,
    actions: Vec<Action>,
    hidden_previews_body_placeholder: Option<String>,
    #[serde(default)]
    custom_dismiss_action: bool,
    #[serde(default)]
    allow_in_car_play: bool,
    #[serde(default)]
    hidden_previews_show_title: bool,
    #[serde(default)]
    hidden_previews_show_subtitle: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    id: String,
    title: String,
    #[serde(default)]
    requires_authentication: bool,
    #[serde(default)]
    foreground: bool,
    #[serde(default)]
    destructive: bool,
    #[serde(default)]
    input: bool,
    input_button_title: Option<String>,
    input_placeholder: Option<String>,
}

pub use android::*;

mod android {
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

    #[derive(Debug, Default, Clone, Copy, Serialize_repr, Deserialize_repr)]
    #[repr(u8)]
    pub enum Importance {
        None = 0,
        Min = 1,
        Low = 2,
        #[default]
        Default = 3,
        High = 4,
    }

    #[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
    #[repr(i8)]
    pub enum Visibility {
        Secret = -1,
        Private = 0,
        Public = 1,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Channel {
        id: String,
        name: String,
        description: Option<String>,
        sound: Option<String>,
        lights: Option<bool>,
        light_color: Option<String>,
        vibration: Option<bool>,
        importance: Option<Importance>,
        visibility: Option<Visibility>,
    }

    #[derive(Debug)]
    pub struct ChannelBuilder(Channel);

    impl Channel {
        pub fn builder(id: impl Into<String>, name: impl Into<String>) -> ChannelBuilder {
            ChannelBuilder(Self {
                id: id.into(),
                name: name.into(),
                description: None,
                sound: None,
                lights: Some(false),
                light_color: None,
                vibration: Some(false),
                importance: Default::default(),
                visibility: None,
            })
        }

        pub fn id(&self) -> &str {
            &self.id
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn description(&self) -> Option<&str> {
            self.description.as_deref()
        }

        pub fn sound(&self) -> Option<&str> {
            self.sound.as_deref()
        }

        pub fn lights(&self) -> bool {
            self.lights.unwrap_or(false)
        }

        pub fn light_color(&self) -> Option<&str> {
            self.light_color.as_deref()
        }

        pub fn vibration(&self) -> bool {
            self.vibration.unwrap_or(false)
        }

        pub fn importance(&self) -> Importance {
            self.importance.unwrap_or_default()
        }

        pub fn visibility(&self) -> Option<Visibility> {
            self.visibility
        }
    }

    impl ChannelBuilder {
        pub fn description(mut self, description: impl Into<String>) -> Self {
            self.0.description.replace(description.into());
            self
        }

        pub fn sound(mut self, sound: impl Into<String>) -> Self {
            self.0.sound.replace(sound.into());
            self
        }

        pub fn lights(mut self, lights: bool) -> Self {
            self.0.lights = Some(lights);
            self
        }

        pub fn light_color(mut self, color: impl Into<String>) -> Self {
            self.0.light_color.replace(color.into());
            self
        }

        pub fn vibration(mut self, vibration: bool) -> Self {
            self.0.vibration = Some(vibration);
            self
        }

        pub fn importance(mut self, importance: Importance) -> Self {
            self.0.importance = Some(importance);
            self
        }

        pub fn visibility(mut self, visibility: Visibility) -> Self {
            self.0.visibility.replace(visibility);
            self
        }

        pub fn build(self) -> Channel {
            self.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_creation() {
        let url = Url::parse("https://example.com/image.png").expect("Failed to parse URL");
        let attachment = Attachment::new("test_id", url.clone());
        assert_eq!(attachment.id, "test_id");
        assert_eq!(attachment.url, url);
    }

    #[test]
    fn test_attachment_serialization() {
        let url = Url::parse("https://example.com/image.png").expect("Failed to parse URL");
        let attachment = Attachment::new("test_id", url);
        let json = serde_json::to_string(&attachment).expect("Failed to serialize attachment");
        assert!(json.contains("test_id"));
        assert!(json.contains("https://example.com/image.png"));
    }

    #[test]
    fn test_attachment_deserialization() {
        let json = r#"{"id":"test_id","url":"https://example.com/image.png"}"#;
        let attachment: Attachment =
            serde_json::from_str(json).expect("Failed to deserialize attachment");
        assert_eq!(attachment.id, "test_id");
        assert_eq!(attachment.url.as_str(), "https://example.com/image.png");
    }

    #[test]
    fn test_schedule_every_display() {
        assert_eq!(ScheduleEvery::Year.to_string(), "year");
        assert_eq!(ScheduleEvery::Month.to_string(), "month");
        assert_eq!(ScheduleEvery::TwoWeeks.to_string(), "twoWeeks");
        assert_eq!(ScheduleEvery::Week.to_string(), "week");
        assert_eq!(ScheduleEvery::Day.to_string(), "day");
        assert_eq!(ScheduleEvery::Hour.to_string(), "hour");
        assert_eq!(ScheduleEvery::Minute.to_string(), "minute");
        assert_eq!(ScheduleEvery::Second.to_string(), "second");
    }

    #[test]
    fn test_schedule_every_serialization() {
        let json = serde_json::to_string(&ScheduleEvery::Day).expect("Failed to serialize Day");
        assert_eq!(json, "\"day\"");

        let json =
            serde_json::to_string(&ScheduleEvery::TwoWeeks).expect("Failed to serialize TwoWeeks");
        assert_eq!(json, "\"twoWeeks\"");
    }

    #[test]
    fn test_schedule_every_deserialization() {
        let every: ScheduleEvery =
            serde_json::from_str("\"year\"").expect("Failed to deserialize year");
        assert!(matches!(every, ScheduleEvery::Year));

        let every: ScheduleEvery =
            serde_json::from_str("\"month\"").expect("Failed to deserialize month");
        assert!(matches!(every, ScheduleEvery::Month));

        let every: ScheduleEvery =
            serde_json::from_str("\"twoweeks\"").expect("Failed to deserialize twoweeks");
        assert!(matches!(every, ScheduleEvery::TwoWeeks));

        let every: ScheduleEvery =
            serde_json::from_str("\"week\"").expect("Failed to deserialize week");
        assert!(matches!(every, ScheduleEvery::Week));

        let every: ScheduleEvery =
            serde_json::from_str("\"day\"").expect("Failed to deserialize day");
        assert!(matches!(every, ScheduleEvery::Day));

        let every: ScheduleEvery =
            serde_json::from_str("\"hour\"").expect("Failed to deserialize hour");
        assert!(matches!(every, ScheduleEvery::Hour));

        let every: ScheduleEvery =
            serde_json::from_str("\"minute\"").expect("Failed to deserialize minute");
        assert!(matches!(every, ScheduleEvery::Minute));

        let every: ScheduleEvery =
            serde_json::from_str("\"second\"").expect("Failed to deserialize second");
        assert!(matches!(every, ScheduleEvery::Second));
    }

    #[test]
    fn test_schedule_every_deserialization_invalid() {
        let result: Result<ScheduleEvery, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_schedule_interval_default() {
        let interval = ScheduleInterval::default();
        assert!(interval.year.is_none());
        assert!(interval.month.is_none());
        assert!(interval.day.is_none());
        assert!(interval.weekday.is_none());
        assert!(interval.hour.is_none());
        assert!(interval.minute.is_none());
        assert!(interval.second.is_none());
    }

    #[test]
    fn test_schedule_interval_serialization() {
        let interval = ScheduleInterval {
            year: Some(24),
            month: Some(12),
            day: Some(25),
            weekday: Some(1),
            hour: Some(10),
            minute: Some(30),
            second: Some(0),
        };
        let json = serde_json::to_string(&interval).expect("Failed to serialize interval");
        assert!(json.contains("\"year\":24"));
        assert!(json.contains("\"month\":12"));
        assert!(json.contains("\"day\":25"));
    }

    #[test]
    fn test_notification_data_default() {
        let data = NotificationData::default();
        assert!(data.id != 0); // Should be a random ID
        assert!(data.channel_id.is_none());
        assert!(data.title.is_none());
        assert!(data.body.is_none());
        assert!(data.schedule.is_none());
        assert!(!data.group_summary);
        assert!(!data.ongoing);
        assert!(!data.auto_cancel);
        assert!(!data.silent);
        assert!(data.inbox_lines.is_empty());
        assert!(data.attachments.is_empty());
        assert!(data.extra.is_empty());
    }

    #[test]
    fn test_notification_data_serialization() {
        let data = NotificationData {
            id: 123,
            title: Some("Test Title".to_string()),
            body: Some("Test Body".to_string()),
            ongoing: true,
            ..Default::default()
        };

        let json = serde_json::to_string(&data).expect("Failed to serialize notification data");
        assert!(json.contains("\"id\":123"));
        assert!(json.contains("\"title\":\"Test Title\""));
        assert!(json.contains("\"body\":\"Test Body\""));
        assert!(json.contains("\"ongoing\":true"));
    }

    #[test]
    fn test_pending_notification_getters() {
        let json = r#"{
            "id": 456,
            "title": "Pending Title",
            "body": "Pending Body",
            "schedule": {"every": {"interval": "day", "count": 1}}
        }"#;
        let pending: PendingNotification =
            serde_json::from_str(json).expect("Failed to deserialize pending notification");

        assert_eq!(pending.id(), 456);
        assert_eq!(pending.title(), Some("Pending Title"));
        assert_eq!(pending.body(), Some("Pending Body"));
        assert!(matches!(pending.schedule(), Schedule::Every { .. }));
    }

    #[test]
    fn test_active_notification_getters() {
        let json = r#"{
            "id": 789,
            "title": "Active Title",
            "body": "Active Body",
            "group": "test_group",
            "groupSummary": true
        }"#;
        let active: ActiveNotification =
            serde_json::from_str(json).expect("Failed to deserialize active notification");

        assert_eq!(active.id(), 789);
        assert_eq!(active.title(), Some("Active Title"));
        assert_eq!(active.body(), Some("Active Body"));
        assert_eq!(active.group(), Some("test_group"));
        assert!(active.group_summary());
        assert!(active.data().is_empty());
        assert!(active.extra().is_empty());
        assert!(active.attachments().is_empty());
        assert!(active.action_type_id().is_none());
        assert!(active.schedule().is_none());
        assert!(active.sound().is_none());
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_importance_default() {
        let importance = Importance::default();
        assert!(matches!(importance, Importance::Default));
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_importance_serialization() {
        assert_eq!(
            serde_json::to_string(&Importance::None).expect("Failed to serialize Importance::None"),
            "0"
        );
        assert_eq!(
            serde_json::to_string(&Importance::Min).expect("Failed to serialize Importance::Min"),
            "1"
        );
        assert_eq!(
            serde_json::to_string(&Importance::Low).expect("Failed to serialize Importance::Low"),
            "2"
        );
        assert_eq!(
            serde_json::to_string(&Importance::Default)
                .expect("Failed to serialize Importance::Default"),
            "3"
        );
        assert_eq!(
            serde_json::to_string(&Importance::High).expect("Failed to serialize Importance::High"),
            "4"
        );
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_visibility_serialization() {
        assert_eq!(
            serde_json::to_string(&Visibility::Secret)
                .expect("Failed to serialize Visibility::Secret"),
            "-1"
        );
        assert_eq!(
            serde_json::to_string(&Visibility::Private)
                .expect("Failed to serialize Visibility::Private"),
            "0"
        );
        assert_eq!(
            serde_json::to_string(&Visibility::Public)
                .expect("Failed to serialize Visibility::Public"),
            "1"
        );
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_channel_builder() {
        let channel = Channel::builder("test_id", "Test Channel")
            .description("Test Description")
            .sound("test_sound")
            .lights(true)
            .light_color("#FF0000")
            .vibration(true)
            .importance(Importance::High)
            .visibility(Visibility::Public)
            .build();

        assert_eq!(channel.id(), "test_id");
        assert_eq!(channel.name(), "Test Channel");
        assert_eq!(channel.description(), Some("Test Description"));
        assert_eq!(channel.sound(), Some("test_sound"));
        assert!(channel.lights());
        assert_eq!(channel.light_color(), Some("#FF0000"));
        assert!(channel.vibration());
        assert!(matches!(channel.importance(), Importance::High));
        assert_eq!(channel.visibility(), Some(Visibility::Public));
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_channel_builder_minimal() {
        let channel = Channel::builder("minimal_id", "Minimal Channel").build();

        assert_eq!(channel.id(), "minimal_id");
        assert_eq!(channel.name(), "Minimal Channel");
        assert_eq!(channel.description(), None);
        assert_eq!(channel.sound(), None);
        assert!(!channel.lights());
        assert_eq!(channel.light_color(), None);
        assert!(!channel.vibration());
        assert!(matches!(channel.importance(), Importance::Default));
        assert_eq!(channel.visibility(), None);
    }

    #[test]
    fn test_schedule_at_serialization() {
        use time::OffsetDateTime;

        let date = OffsetDateTime::now_utc();
        let schedule = Schedule::At {
            date,
            repeating: true,
            allow_while_idle: false,
        };

        let json = serde_json::to_string(&schedule).expect("Failed to serialize Schedule::At");
        assert!(json.contains("\"at\""));
        assert!(json.contains("\"date\""));
        assert!(json.contains("\"repeating\":true"));
        assert!(json.contains("\"allowWhileIdle\":false"));
    }

    #[test]
    fn test_schedule_interval_variant() {
        let schedule = Schedule::Interval {
            interval: ScheduleInterval {
                hour: Some(10),
                minute: Some(30),
                ..Default::default()
            },
            allow_while_idle: true,
        };

        let json =
            serde_json::to_string(&schedule).expect("Failed to serialize Schedule::Interval");
        assert!(json.contains("\"interval\""));
        assert!(json.contains("\"hour\":10"));
        assert!(json.contains("\"minute\":30"));
        assert!(json.contains("\"allowWhileIdle\":true"));
    }

    #[test]
    fn test_schedule_every_variant() {
        let schedule = Schedule::Every {
            interval: ScheduleEvery::Day,
            count: 5,
            allow_while_idle: false,
        };

        let json = serde_json::to_string(&schedule).expect("Failed to serialize Schedule::Every");
        assert!(json.contains("\"every\""));
        assert!(json.contains("\"interval\":\"day\""));
        assert!(json.contains("\"count\":5"));
    }
}
