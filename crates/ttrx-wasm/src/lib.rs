#![forbid(unsafe_code)]

use std::fmt::{self, Write as _};

use ttrx_core::{ContainerInfo, SourceKind, SourceSummary, Verification};
use wasm_bindgen::prelude::*;

/// Encode one single-player TETR.IO replay JSON document.
///
/// # Errors
///
/// Returns a JavaScript error when the bytes are invalid JSON, do not contain a
/// single-player replay, or cannot be represented by the TTRX codec.
#[wasm_bindgen]
pub fn encode_ttr(source: &[u8]) -> Result<Vec<u8>, JsValue> {
    encode_with_kind(source, SourceKind::Ttr)
}

/// Encode one multiplayer TETR.IO replay JSON document.
///
/// # Errors
///
/// Returns a JavaScript error when the bytes are invalid JSON, do not contain a
/// multiplayer replay, or cannot be represented by the TTRX codec.
#[wasm_bindgen]
pub fn encode_ttrm(source: &[u8]) -> Result<Vec<u8>, JsValue> {
    encode_with_kind(source, SourceKind::Ttrm)
}

/// Decode a TTRX container to canonical TETR.IO replay JSON bytes.
///
/// # Errors
///
/// Returns a JavaScript error when the container header, checksum, payload, or
/// decoded JSON data model is invalid.
#[wasm_bindgen]
pub fn decode_ttrx(container: &[u8]) -> Result<Vec<u8>, JsValue> {
    ttrx_core::decode(container)
        .map(|(source, _)| source)
        .map_err(|error| js_error("cannot decode TTRX", &error))
}

/// Return the source extension recorded by a TTRX container (`ttr`, `ttrm`,
/// or `json` when the source kind is unknown).
///
/// # Errors
///
/// Returns a JavaScript error when the container is invalid.
#[wasm_bindgen]
pub fn ttrx_source_extension(container: &[u8]) -> Result<String, JsValue> {
    ttrx_core::codec::decode(container)
        .map(|decoded| {
            decoded
                .info
                .source_kind
                .extension()
                .unwrap_or("json")
                .to_owned()
        })
        .map_err(|error| js_error("cannot read TTRX source extension", &error))
}

/// Describe a single-player source replay without executing game rules.
///
/// # Errors
///
/// Returns a JavaScript error when the source is invalid JSON or its structure
/// is incompatible with a single-player replay.
#[wasm_bindgen]
pub fn inspect_ttr(source: &[u8]) -> Result<String, JsValue> {
    inspect_source_with_kind(source, SourceKind::Ttr)
}

/// Describe a multiplayer source replay without executing game rules.
///
/// # Errors
///
/// Returns a JavaScript error when the source is invalid JSON or its structure
/// is incompatible with a multiplayer replay.
#[wasm_bindgen]
pub fn inspect_ttrm(source: &[u8]) -> Result<String, JsValue> {
    inspect_source_with_kind(source, SourceKind::Ttrm)
}

/// Validate and describe both the binary container and its decoded replay.
///
/// # Errors
///
/// Returns a JavaScript error when the container or decoded replay is invalid.
#[wasm_bindgen]
pub fn inspect_ttrx(container: &[u8]) -> Result<String, JsValue> {
    let (source, info) =
        ttrx_core::decode(container).map_err(|error| js_error("cannot inspect TTRX", &error))?;
    let summary = ttrx_core::inspect_source(&source, info.source_kind)
        .map_err(|error| js_error("cannot inspect decoded replay", &error))?;
    Ok(format!(
        "{}\n\n{}",
        format_container(&info),
        format_summary(&summary)
    ))
}

/// Verify a single-player source replay through an encode/decode round trip.
///
/// # Errors
///
/// Returns a JavaScript error when validation, encoding, decoding, or semantic
/// data-model comparison fails.
#[wasm_bindgen]
pub fn verify_ttr(source: &[u8]) -> Result<String, JsValue> {
    verify_source_with_kind(source, SourceKind::Ttr)
}

/// Verify a multiplayer source replay through an encode/decode round trip.
///
/// # Errors
///
/// Returns a JavaScript error when validation, encoding, decoding, or semantic
/// data-model comparison fails.
#[wasm_bindgen]
pub fn verify_ttrm(source: &[u8]) -> Result<String, JsValue> {
    verify_source_with_kind(source, SourceKind::Ttrm)
}

/// Validate a binary container and verify its decoded replay data model.
///
/// # Errors
///
/// Returns a JavaScript error when container validation or the decoded replay's
/// encode/decode verification fails.
#[wasm_bindgen]
pub fn verify_ttrx(container: &[u8]) -> Result<String, JsValue> {
    let (source, info) =
        ttrx_core::decode(container).map_err(|error| js_error("cannot verify TTRX", &error))?;
    let verification = ttrx_core::verify(&source, info.source_kind)
        .map_err(|error| js_error("cannot verify decoded replay", &error))?;
    Ok(format!(
        "{}\n\n{}",
        format_container(&info),
        format_verification(&verification)
    ))
}

/// Return the converter package version.
#[wasm_bindgen]
#[must_use]
pub fn ttrx_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

fn encode_with_kind(source: &[u8], kind: SourceKind) -> Result<Vec<u8>, JsValue> {
    ttrx_core::encode(source, kind).map_err(|error| js_error("cannot encode replay", &error))
}

fn inspect_source_with_kind(source: &[u8], kind: SourceKind) -> Result<String, JsValue> {
    ttrx_core::inspect_source(source, kind)
        .map(|summary| format_summary(&summary))
        .map_err(|error| js_error("cannot inspect replay", &error))
}

fn verify_source_with_kind(source: &[u8], kind: SourceKind) -> Result<String, JsValue> {
    ttrx_core::verify(source, kind)
        .map(|verification| format_verification(&verification))
        .map_err(|error| js_error("cannot verify replay", &error))
}

fn js_error(context: &str, error: &impl fmt::Display) -> JsValue {
    JsValue::from_str(&format!("{context}: {error}"))
}

fn format_container(info: &ContainerInfo) -> String {
    format!(
        "TTRX format: {}.{}\nProfile: {}\nSource kind: {}\nOriginal JSON bytes: {}\nPayload bytes: {}\nPayload CRC32C: {:08x}\nDictionary entries: {}\nShape entries: {}\nEncoded values: {}\nMaximum depth: {}",
        info.format_major,
        info.format_minor,
        info.profile,
        info.source_kind,
        info.original_json_len,
        info.payload_len,
        info.payload_crc32c,
        info.dictionary_entries,
        info.shape_entries,
        info.value_count,
        info.max_depth
    )
}

fn format_summary(summary: &SourceSummary) -> String {
    let mut output = String::new();
    let container_version = summary.container_version.as_deref().unwrap_or("<missing>");
    let rules_versions = if summary.rules_versions.is_empty() {
        "<missing>".to_owned()
    } else {
        summary.rules_versions.join(", ")
    };
    let _ = writeln!(output, "Source kind: {}", summary.kind);
    let _ = writeln!(output, "Container version: {container_version}");
    let _ = writeln!(output, "Rules versions: {rules_versions}");
    let _ = writeln!(output, "Replay streams: {}", summary.stream_count);
    let _ = writeln!(output, "Events: {}", summary.event_count);
    let _ = writeln!(output, "Key events: {}", summary.key_event_count());
    let _ = writeln!(output, "IGE events: {}", summary.ige_event_count());
    output.push_str("Event types: ");
    if summary.event_types.is_empty() {
        output.push_str("<none>");
    } else {
        join_map(
            &mut output,
            summary
                .event_types
                .iter()
                .map(|(name, count)| (name.as_str(), *count)),
        );
    }
    let _ = write!(output, "\nOption keys ({}): ", summary.option_keys.len());
    if summary.option_keys.is_empty() {
        output.push_str("<none>");
    } else {
        output.push_str(&summary.option_keys.join(", "));
    }
    output
}

fn join_map<'a>(output: &mut String, values: impl Iterator<Item = (&'a str, usize)>) {
    for (index, (name, count)) in values.enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "{name}={count}");
    }
}

fn format_verification(verification: &Verification) -> String {
    format!(
        "Verification: passed\nSource kind: {}\nReplay streams: {}\nEvents: {}\nInput JSON bytes: {}\nTTRX bytes: {}\nCanonical JSON bytes: {}\nExact JSON data model: {}",
        verification.summary.kind,
        verification.summary.stream_count,
        verification.summary.event_count,
        verification.input_bytes,
        verification.encoded_bytes,
        verification.canonical_json_bytes,
        verification.exact_data_model
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{format_summary, ttrx_source_extension};
    use ttrx_core::{SourceKind, SourceSummary};

    #[test]
    fn summary_text_is_stable_and_human_readable() {
        let summary = SourceSummary {
            kind: SourceKind::Ttr,
            container_version: Some("1".into()),
            rules_versions: vec!["19".into()],
            stream_count: 1,
            event_count: 2,
            event_types: BTreeMap::from([("end".into(), 1), ("start".into(), 1)]),
            option_keys: vec!["g".into(), "version".into()],
        };
        let text = format_summary(&summary);
        assert!(text.contains("Source kind: ttr"));
        assert!(text.contains("Event types: end=1, start=1"));
        assert!(text.contains("Option keys (2): g, version"));
    }

    #[test]
    fn exposes_the_recorded_source_extension() {
        let source = br#"{"replay":{"events":[{"type":"start"}]}}"#;
        let container = ttrx_core::encode(source, SourceKind::Ttr).unwrap();
        assert_eq!(ttrx_source_extension(&container).unwrap(), "ttr");
    }
}
