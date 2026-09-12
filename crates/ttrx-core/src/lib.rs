#![forbid(unsafe_code)]

pub mod codec;
pub mod crc32c;
pub mod json;
pub mod source;
pub mod varint;

use std::error::Error as StdError;
use std::fmt;

pub use codec::{ContainerInfo, DecodedDocument, FormatProfile, SourceKind};
pub use source::{SourceSummary, Verification};

#[derive(Debug)]
pub enum Error {
    Json(json::JsonError),
    Codec(codec::CodecError),
    Source(source::SourceError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "JSON error: {error}"),
            Self::Codec(error) => write!(f, "TTRX error: {error}"),
            Self::Source(error) => write!(f, "source error: {error}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Codec(error) => Some(error),
            Self::Source(error) => Some(error),
        }
    }
}

impl From<json::JsonError> for Error {
    fn from(value: json::JsonError) -> Self {
        Self::Json(value)
    }
}

impl From<codec::CodecError> for Error {
    fn from(value: codec::CodecError) -> Self {
        Self::Codec(value)
    }
}

impl From<source::SourceError> for Error {
    fn from(value: source::SourceError) -> Self {
        Self::Source(value)
    }
}

/// Parse a TETR.IO JSON replay and encode it as a source-semantic TTRX container.
///
/// # Errors
///
/// Returns an error for invalid JSON, an unsupported replay shape, a conflicting
/// source-kind hint, resource-limit violations, or an encoding failure.
pub fn encode(source_json: &[u8], hint: SourceKind) -> Result<Vec<u8>, Error> {
    let value = json::parse(source_json)?;
    let detected = source::validate_and_detect(&value, hint)?;
    Ok(codec::encode(&value, detected, source_json.len() as u64)?)
}

/// Decode a TTRX container to canonical JSON and retain its container metadata.
///
/// # Errors
///
/// Returns an error for an unsupported, corrupt, truncated, oversized, or
/// otherwise malformed TTRX container.
pub fn decode(container: &[u8]) -> Result<(Vec<u8>, ContainerInfo), Error> {
    let decoded = codec::decode(container)?;
    let encoded_len = json::encoded_len(&decoded.value).ok_or(codec::CodecError::LengthOverflow)?;
    if encoded_len > json::MAX_JSON_BYTES {
        return Err(codec::CodecError::LimitExceeded("decoded JSON length").into());
    }
    let json = json::to_vec(&decoded.value);
    debug_assert_eq!(json.len(), encoded_len);
    Ok((json, decoded.info))
}

/// Inspect a source replay without executing its game rules.
///
/// # Errors
///
/// Returns an error for invalid JSON, an unsupported replay shape, a conflicting
/// source-kind hint, or source resource-limit violations.
pub fn inspect_source(source_json: &[u8], hint: SourceKind) -> Result<SourceSummary, Error> {
    let value = json::parse(source_json)?;
    Ok(source::summarize(&value, hint)?)
}

/// Verify the binary round trip against the parsed JSON data model.
///
/// # Errors
///
/// Returns an error when source parsing or structural validation fails, the
/// container cannot be encoded/decoded, or the decoded data model differs.
pub fn verify(source_json: &[u8], hint: SourceKind) -> Result<Verification, Error> {
    let original = json::parse(source_json)?;
    let summary = source::summarize(&original, hint)?;
    let encoded = codec::encode(&original, summary.kind, source_json.len() as u64)?;
    let decoded = codec::decode(&encoded)?;
    if decoded.value != original {
        return Err(source::SourceError::SemanticMismatch.into());
    }
    let canonical = json::to_vec(&decoded.value);
    Ok(Verification {
        summary,
        input_bytes: source_json.len(),
        encoded_bytes: encoded.len(),
        canonical_json_bytes: canonical.len(),
        exact_data_model: true,
    })
}
