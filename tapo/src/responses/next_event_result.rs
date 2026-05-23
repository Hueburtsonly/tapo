use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::responses::TapoResponseExt;

/// The next event the device is scheduled to act on — typically a
/// schedule rule firing.  Returned by
/// [`PlugHandler::get_next_event`](crate::PlugHandler::get_next_event)
/// and the equivalent method on
/// [`PlugEnergyMonitoringHandler`](crate::PlugEnergyMonitoringHandler).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass(from_py_object, get_all))]
pub struct NextEvent {
    /// Device-assigned id of the schedule rule that will fire next.
    pub id: String,
    /// When the event fires (UTC).
    pub start_time: DateTime<Utc>,
    /// Optional end-time for two-stage rules.  `None` when the rule
    /// has no end action (the common case for plug schedules).
    pub end_time: Option<DateTime<Utc>>,
    /// Wire-level event type code as the device reports it.  Value `1`
    /// has been observed for plug schedule rules; the field is kept
    /// for forward compatibility with other event kinds.
    pub event_type: i32,
    /// Whether the device will turn on (`true`) or off (`false`) when
    /// the event fires.
    pub turn_on: bool,
}

#[cfg(feature = "python")]
crate::impl_to_dict!(NextEvent);

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawNextEvent {
    // All fields are optional: when the plug has no armed rule the device
    // still returns a `{}`-shaped result, which we want to map to `None`
    // rather than a deserialization error.
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub s_time: Option<i64>,
    #[serde(default)]
    pub e_time: i64,
    #[serde(rename = "type", default)]
    pub event_type: i32,
    #[serde(default)]
    pub desired_states: Option<serde_json::Value>,
}

impl TapoResponseExt for RawNextEvent {}

impl RawNextEvent {
    pub(crate) fn into_next_event(self) -> Option<NextEvent> {
        let id = self.id?;
        let start_time = DateTime::<Utc>::from_timestamp(self.s_time?, 0)?;
        let end_time = if self.e_time == 0 {
            None
        } else {
            DateTime::<Utc>::from_timestamp(self.e_time, 0)
        };
        let turn_on = self
            .desired_states
            .as_ref()
            .and_then(|v| v.get("on").and_then(|x| x.as_bool()))?;
        Some(NextEvent {
            id,
            start_time,
            end_time,
            event_type: self.event_type,
            turn_on,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_to_next_event_basic() {
        let raw: RawNextEvent = serde_json::from_value(serde_json::json!({
            "id": "S1",
            "s_time": 1_779_599_760_i64,
            "e_time": 0,
            "type": 1,
            "desired_states": { "on": true },
        }))
        .unwrap();
        let n = raw.into_next_event().unwrap();
        assert_eq!(n.id, "S1");
        assert_eq!(n.start_time.timestamp(), 1_779_599_760);
        assert!(n.end_time.is_none());
        assert_eq!(n.event_type, 1);
        assert!(n.turn_on);
    }

    #[test]
    fn turn_off_decodes() {
        let raw: RawNextEvent = serde_json::from_value(serde_json::json!({
            "id": "S2", "s_time": 100, "type": 1,
            "desired_states": { "on": false },
        }))
        .unwrap();
        assert!(!raw.into_next_event().unwrap().turn_on);
    }

    #[test]
    fn end_time_nonzero_decodes() {
        let raw: RawNextEvent = serde_json::from_value(serde_json::json!({
            "id": "S3", "s_time": 100, "e_time": 200, "type": 1,
            "desired_states": { "on": true },
        }))
        .unwrap();
        assert_eq!(
            raw.into_next_event().unwrap().end_time.unwrap().timestamp(),
            200
        );
    }

    #[test]
    fn missing_desired_states_yields_none() {
        // No firing state recoverable — return None so a future field can be
        // added without us silently inventing a value.
        let raw: RawNextEvent =
            serde_json::from_value(serde_json::json!({ "id": "S4", "s_time": 1 })).unwrap();
        assert!(raw.into_next_event().is_none());
    }

    #[test]
    fn empty_object_yields_none() {
        // Real behavior observed on a P110: when no rule is armed the device
        // returns an empty `{}` result rather than omitting the response.
        let raw: RawNextEvent = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(raw.into_next_event().is_none());
    }
}
