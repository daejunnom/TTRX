use std::collections::{BTreeMap, BTreeSet};
use std::error::Error as StdError;
use std::fmt;

use crate::codec::SourceKind;
use crate::json::JsonValue;

const MAX_STREAMS: usize = 100_000;
const MAX_EVENTS: usize = 20_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSummary {
    pub kind: SourceKind,
    pub container_version: Option<String>,
    pub rules_versions: Vec<String>,
    pub stream_count: usize,
    pub event_count: usize,
    pub event_types: BTreeMap<String, usize>,
    pub option_keys: Vec<String>,
}

impl SourceSummary {
    #[must_use]
    pub fn key_event_count(&self) -> usize {
        self.event_types.get("keydown").copied().unwrap_or(0)
            + self.event_types.get("keyup").copied().unwrap_or(0)
    }

    #[must_use]
    pub fn ige_event_count(&self) -> usize {
        self.event_types.get("ige").copied().unwrap_or(0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verification {
    pub summary: SourceSummary,
    pub input_bytes: usize,
    pub encoded_bytes: usize,
    pub canonical_json_bytes: usize,
    pub exact_data_model: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceError {
    RootMustBeObject,
    UnsupportedShape,
    InvalidStructure(&'static str),
    KindMismatch {
        requested: SourceKind,
        detected: SourceKind,
    },
    TooManyStreams,
    TooManyEvents,
    SemanticMismatch,
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeObject => {
                write!(f, "a TETR.IO replay must have a JSON object at its root")
            }
            Self::UnsupportedShape => write!(
                f,
                "no replay event stream was found in the supported TTR/TTRM structures"
            ),
            Self::InvalidStructure(reason) => {
                write!(f, "invalid TETR.IO replay structure: {reason}")
            }
            Self::KindMismatch {
                requested,
                detected,
            } => write!(
                f,
                "source kind hint {requested} conflicts with detected replay shape {detected}"
            ),
            Self::TooManyStreams => write!(f, "replay stream count exceeds the safety limit"),
            Self::TooManyEvents => write!(f, "replay event count exceeds the safety limit"),
            Self::SemanticMismatch => write!(f, "decoded JSON data model differs from the source"),
        }
    }
}

impl StdError for SourceError {}

/// Validate a parsed replay structure and resolve its single/multi source kind.
///
/// # Errors
///
/// Returns an error for a non-object or unsupported replay shape, a conflicting
/// explicit kind, or a document that exceeds the stream/event limits.
pub fn validate_and_detect(value: &JsonValue, hint: SourceKind) -> Result<SourceKind, SourceError> {
    Ok(summarize(value, hint)?.kind)
}

/// Collect structural metadata without executing TETR.IO game rules.
///
/// # Errors
///
/// Returns an error for a non-object or unsupported replay shape, a conflicting
/// explicit kind, or a document that exceeds the stream/event limits.
pub fn summarize(value: &JsonValue, hint: SourceKind) -> Result<SourceSummary, SourceError> {
    let root = object(value).ok_or(SourceError::RootMustBeObject)?;
    let mut streams = Vec::new();
    collect_known_streams(root, &mut streams)?;
    if streams.is_empty() {
        return Err(SourceError::UnsupportedShape);
    }
    if streams.len() > MAX_STREAMS {
        return Err(SourceError::TooManyStreams);
    }

    let structurally_multi = has_multi_shape(root) || streams.len() > 1;
    let detected = if structurally_multi {
        SourceKind::Ttrm
    } else {
        SourceKind::Ttr
    };
    let kind = match hint {
        SourceKind::Unknown => detected,
        requested if requested == detected => requested,
        // A .ttrm may contain one stream (for example a truncated export), so an
        // explicit multi hint is accepted. A .ttr hint must not hide a known
        // multi-round structure.
        SourceKind::Ttrm if detected == SourceKind::Ttr => SourceKind::Ttrm,
        requested => {
            return Err(SourceError::KindMismatch {
                requested,
                detected,
            });
        }
    };

    let mut event_count = 0_usize;
    let mut event_types = BTreeMap::new();
    let mut rules_versions = BTreeSet::new();
    let mut option_keys = BTreeSet::new();
    for stream in &streams {
        let entries = array(member(stream, "events").ok_or(SourceError::UnsupportedShape)?)
            .ok_or(SourceError::UnsupportedShape)?;
        event_count = event_count
            .checked_add(entries.len())
            .ok_or(SourceError::TooManyEvents)?;
        if event_count > MAX_EVENTS {
            return Err(SourceError::TooManyEvents);
        }
        for event in entries {
            let Some(event) = object(event) else { continue };
            let event_type = member(event, "type").and_then(string);
            let name = event_type.unwrap_or("<non-string>").to_owned();
            *event_types.entry(name).or_insert(0) += 1;
        }
        if let Some(options) = member(stream, "options").and_then(object) {
            for (key, _) in options {
                option_keys.insert(key.clone());
            }
            if let Some(version) = member(options, "version").and_then(scalar_text) {
                rules_versions.insert(version);
            }
        }
    }

    Ok(SourceSummary {
        kind,
        container_version: member(root, "version").and_then(scalar_text),
        rules_versions: rules_versions.into_iter().collect(),
        stream_count: streams.len(),
        event_count,
        event_types,
        option_keys: option_keys.into_iter().collect(),
    })
}

fn collect_known_streams<'a>(
    root: &'a [(String, JsonValue)],
    output: &mut Vec<&'a [(String, JsonValue)]>,
) -> Result<(), SourceError> {
    if let Some(replay) = member(root, "replay").and_then(object) {
        if is_stream(replay) {
            output.push(replay);
            return Ok(());
        }
        if let Some(rounds) = member(replay, "rounds") {
            let rounds = array(rounds).ok_or(SourceError::InvalidStructure(
                "replay.rounds must be an array",
            ))?;
            collect_rounds(rounds, output)?;
            return Ok(());
        }
    }

    // Older single-player candidate documented by the handoff.
    if let Some(data) = member(root, "data").and_then(object) {
        if is_stream(data) {
            output.push(data);
            return Ok(());
        }
    }

    // Older multi-player exports have appeared as data[r].replays[p].events.
    if let Some(rounds) = member(root, "data").and_then(array) {
        collect_legacy_rounds(rounds, output)?;
    }
    Ok(())
}

fn collect_legacy_rounds<'a>(
    rounds: &'a [JsonValue],
    output: &mut Vec<&'a [(String, JsonValue)]>,
) -> Result<(), SourceError> {
    for round in rounds {
        let round = object(round).ok_or(SourceError::InvalidStructure(
            "each legacy data round must be an object",
        ))?;
        let replays =
            member(round, "replays")
                .and_then(array)
                .ok_or(SourceError::InvalidStructure(
                    "each legacy data round must contain a replays array",
                ))?;
        for candidate in replays {
            let candidate = object(candidate).ok_or(SourceError::InvalidStructure(
                "each legacy replay entry must be an object",
            ))?;
            if is_stream(candidate) {
                output.push(candidate);
                continue;
            }
            let replay = member(candidate, "replay").and_then(object).ok_or(
                SourceError::InvalidStructure(
                    "each legacy replay entry must contain an event stream",
                ),
            )?;
            if !is_stream(replay) {
                return Err(SourceError::InvalidStructure(
                    "each legacy replay entry must contain valid events",
                ));
            }
            output.push(replay);
        }
    }
    Ok(())
}

fn collect_rounds<'a>(
    rounds: &'a [JsonValue],
    output: &mut Vec<&'a [(String, JsonValue)]>,
) -> Result<(), SourceError> {
    for round in rounds {
        let players = array(round).ok_or(SourceError::InvalidStructure(
            "each replay.rounds entry must be a player array",
        ))?;
        for player in players {
            let player = object(player).ok_or(SourceError::InvalidStructure(
                "each replay.rounds player must be an object",
            ))?;
            if let Some(replay_value) = member(player, "replay") {
                let replay = object(replay_value).ok_or(SourceError::InvalidStructure(
                    "each player replay must be an object",
                ))?;
                if !is_stream(replay) {
                    return Err(SourceError::InvalidStructure(
                        "each player replay must contain valid events",
                    ));
                }
                output.push(replay);
            } else if is_stream(player) {
                output.push(player);
            } else {
                return Err(SourceError::InvalidStructure(
                    "each player must contain a replay event stream",
                ));
            }
        }
    }
    Ok(())
}

fn has_multi_shape(root: &[(String, JsonValue)]) -> bool {
    member(root, "replay")
        .and_then(object)
        .and_then(|replay| member(replay, "rounds"))
        .is_some()
        || member(root, "data").and_then(array).is_some_and(|rounds| {
            rounds.iter().any(|round| {
                object(round)
                    .and_then(|round| member(round, "replays"))
                    .is_some_and(|replays| matches!(replays, JsonValue::Array(_)))
            })
        })
}

fn is_stream(value: &[(String, JsonValue)]) -> bool {
    let Some(events) = member(value, "events").and_then(array) else {
        return false;
    };
    events.iter().all(|event| {
        object(event)
            .and_then(|event| member(event, "type"))
            .and_then(string)
            .is_some()
    })
}

fn member<'a>(object: &'a [(String, JsonValue)], key: &str) -> Option<&'a JsonValue> {
    // JSON.parse and the TETR.IO client retain the final duplicate key. The
    // source model itself still preserves every pair and its order.
    object
        .iter()
        .rev()
        .find_map(|(candidate, value)| (candidate == key).then_some(value))
}

fn object(value: &JsonValue) -> Option<&[(String, JsonValue)]> {
    match value {
        JsonValue::Object(value) => Some(value),
        _ => None,
    }
}

fn array(value: &JsonValue) -> Option<&[JsonValue]> {
    match value {
        JsonValue::Array(value) => Some(value),
        _ => None,
    }
}

fn string(value: &JsonValue) -> Option<&str> {
    match value {
        JsonValue::String(value) => Some(value),
        _ => None,
    }
}

fn scalar_text(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Number(value) | JsonValue::String(value) => Some(value.clone()),
        JsonValue::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceError, summarize};
    use crate::codec::SourceKind;
    use crate::json::parse;

    #[test]
    fn summarizes_single_replay() {
        let value = parse(br#"{"version":1,"replay":{"frames":4,"options":{"version":19,"g":0.02},"events":[{"type":"start"},{"type":"keydown"},{"type":"keyup"},{"type":"end"}]}}"#).unwrap();
        let summary = summarize(&value, SourceKind::Unknown).unwrap();
        assert_eq!(summary.kind, SourceKind::Ttr);
        assert_eq!(summary.container_version.as_deref(), Some("1"));
        assert_eq!(summary.rules_versions, ["19"]);
        assert_eq!(summary.event_count, 4);
        assert_eq!(summary.key_event_count(), 2);
        assert_eq!(summary.option_keys, ["g", "version"]);
    }

    #[test]
    fn summarizes_rounds_and_keeps_event_counts() {
        let value = parse(br#"{"replay":{"rounds":[[{"replay":{"options":{"version":19},"events":[{"type":"ige"}]}},{"replay":{"options":{"version":19},"events":[{"type":"end"}]}}]]}}"#).unwrap();
        let summary = summarize(&value, SourceKind::Ttrm).unwrap();
        assert_eq!(summary.kind, SourceKind::Ttrm);
        assert_eq!(summary.stream_count, 2);
        assert_eq!(summary.event_count, 2);
        assert_eq!(summary.ige_event_count(), 1);
    }

    #[test]
    fn rejects_multi_shape_hidden_by_single_hint() {
        let value = parse(br#"{"replay":{"rounds":[[{"replay":{"events":[]}}]]}}"#).unwrap();
        assert!(matches!(
            summarize(&value, SourceKind::Ttr),
            Err(SourceError::KindMismatch { .. })
        ));
    }

    #[test]
    fn rejects_arbitrary_json() {
        let value = parse(br#"{"hello":"world"}"#).unwrap();
        assert_eq!(
            summarize(&value, SourceKind::Unknown).unwrap_err(),
            SourceError::UnsupportedShape
        );
    }

    #[test]
    fn rejects_non_event_values_inside_a_stream() {
        let value = parse(br#"{"replay":{"events":[null]}}"#).unwrap();
        assert_eq!(
            summarize(&value, SourceKind::Unknown),
            Err(SourceError::UnsupportedShape)
        );
    }

    #[test]
    fn rejects_partially_malformed_multiplayer_rounds() {
        let value =
            parse(br#"{"replay":{"rounds":[[{"replay":{"events":[{"type":"start"}]}},null]]}}"#)
                .unwrap();
        assert!(matches!(
            summarize(&value, SourceKind::Unknown),
            Err(SourceError::InvalidStructure(_))
        ));
    }

    #[test]
    fn unrelated_data_array_does_not_turn_a_single_replay_into_multi() {
        let value = parse(br#"{"data":[],"replay":{"events":[{"type":"start"}]}}"#).unwrap();
        let summary = summarize(&value, SourceKind::Unknown).unwrap();
        assert_eq!(summary.kind, SourceKind::Ttr);
    }
}
