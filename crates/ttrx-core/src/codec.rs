//! Versioned, dependency-free TTRX source-semantic container codec.
//!
//! Version 1.0 uses a fixed 72-byte header followed by one checksummed payload.
//! The payload starts with a UTF-8 string dictionary, continues with a table of
//! repeated object key shapes, and ends with one recursively encoded JSON value.
//! Object member order and duplicate member names remain part of the data model.

use crate::json::JsonValue;
use crate::{crc32c, varint};
use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;

const MAGIC: &[u8; 4] = b"TTRX";
const FORMAT_MAJOR: u8 = 1;
const FORMAT_MINOR: u8 = 0;
const HEADER_LEN: usize = 72;
const HEADER_LEN_U16: u16 = 72;
const HEADER_CRC_OFFSET: usize = 68;

const PROFILE_SOURCE_SEMANTIC: u8 = 1;

const TAG_NULL: u8 = 0;
const TAG_FALSE: u8 = 1;
const TAG_TRUE: u8 = 2;
const TAG_UNSIGNED_INTEGER: u8 = 3;
const TAG_SIGNED_INTEGER: u8 = 4;
const TAG_RAW_NUMBER: u8 = 5;
const TAG_STRING: u8 = 6;
const TAG_ARRAY: u8 = 7;
const TAG_OBJECT_INLINE: u8 = 8;
const TAG_OBJECT_SHAPE: u8 = 9;

// These bounds keep hostile containers from turning small headers into very
// large allocations. They are format-decoder policy, not TETR.IO rule limits.
const MAX_PAYLOAD_BYTES: u64 = 512 * 1024 * 1024;
const MAX_SOURCE_JSON_BYTES: u64 = 64 * 1024 * 1024;
const MAX_DICTIONARY_ENTRIES: u64 = 1_000_000;
const MAX_DICTIONARY_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SHAPE_ENTRIES: u64 = 1_000_000;
const MAX_SHAPE_KEY_REFERENCES: u64 = 2_000_000;
const MAX_STRING_BYTES: u64 = 16 * 1024 * 1024;
const MAX_MATERIALIZED_TEXT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_COLLECTION_ITEMS: u64 = 1_000_000;
const MAX_VALUE_COUNT: u64 = 2_000_000;
// The JSON parser permits 512 nested collections. With the root represented as
// depth one, a scalar inside the innermost collection has data-model depth 513.
const MAX_DEPTH: u32 = 513;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Unknown,
    Ttr,
    Ttrm,
}

impl SourceKind {
    #[must_use]
    pub const fn extension(self) -> Option<&'static str> {
        match self {
            Self::Unknown => None,
            Self::Ttr => Some("ttr"),
            Self::Ttrm => Some("ttrm"),
        }
    }

    const fn as_u8(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Ttr => 1,
            Self::Ttrm => 2,
        }
    }

    fn from_u8(value: u8) -> Result<Self, CodecError> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Ttr),
            2 => Ok(Self::Ttrm),
            other => Err(CodecError::UnsupportedSourceKind(other)),
        }
    }
}

impl fmt::Display for SourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unknown => "unknown",
            Self::Ttr => "ttr",
            Self::Ttrm => "ttrm",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatProfile {
    SourceSemantic,
}

impl FormatProfile {
    const fn as_u8(self) -> u8 {
        match self {
            Self::SourceSemantic => PROFILE_SOURCE_SEMANTIC,
        }
    }

    fn from_u8(value: u8) -> Result<Self, CodecError> {
        match value {
            PROFILE_SOURCE_SEMANTIC => Ok(Self::SourceSemantic),
            other => Err(CodecError::UnsupportedProfile(other)),
        }
    }
}

impl fmt::Display for FormatProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceSemantic => f.write_str("source-semantic"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerInfo {
    pub format_major: u8,
    pub format_minor: u8,
    pub profile: FormatProfile,
    pub source_kind: SourceKind,
    pub original_json_len: u64,
    pub payload_len: u64,
    pub payload_crc32c: u32,
    pub dictionary_entries: u64,
    pub shape_entries: u64,
    pub value_count: u64,
    pub max_depth: u32,
}

impl fmt::Display for ContainerInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TTRX {}.{} {} {} (payload {} bytes, {} values)",
            self.format_major,
            self.format_minor,
            self.profile,
            self.source_kind,
            self.payload_len,
            self.value_count
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedDocument {
    pub value: JsonValue,
    pub info: ContainerInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    UnexpectedEof,
    BadMagic,
    UnsupportedVersion { major: u8, minor: u8 },
    UnsupportedProfile(u8),
    UnsupportedSourceKind(u8),
    InvalidHeader(&'static str),
    HeaderChecksumMismatch { expected: u32, actual: u32 },
    PayloadChecksumMismatch { expected: u32, actual: u32 },
    InvalidVarint,
    NonCanonicalVarint,
    LengthOverflow,
    LimitExceeded(&'static str),
    InvalidUtf8,
    InvalidNumber,
    InvalidStringReference(u64),
    InvalidShapeReference(u64),
    UnknownValueTag(u8),
    StatisticMismatch(&'static str),
    TrailingData,
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected end of TTRX data"),
            Self::BadMagic => f.write_str("invalid TTRX magic"),
            Self::UnsupportedVersion { major, minor } => {
                write!(f, "unsupported TTRX version {major}.{minor}")
            }
            Self::UnsupportedProfile(profile) => {
                write!(f, "unsupported TTRX format profile {profile}")
            }
            Self::UnsupportedSourceKind(kind) => {
                write!(f, "unsupported TTRX source kind {kind}")
            }
            Self::InvalidHeader(reason) => write!(f, "invalid TTRX header: {reason}"),
            Self::HeaderChecksumMismatch { expected, actual } => write!(
                f,
                "TTRX header checksum mismatch (stored {expected:08x}, computed {actual:08x})"
            ),
            Self::PayloadChecksumMismatch { expected, actual } => write!(
                f,
                "TTRX payload checksum mismatch (stored {expected:08x}, computed {actual:08x})"
            ),
            Self::InvalidVarint => f.write_str("invalid TTRX varint"),
            Self::NonCanonicalVarint => f.write_str("non-canonical TTRX varint"),
            Self::LengthOverflow => f.write_str("TTRX length does not fit this platform"),
            Self::LimitExceeded(limit) => write!(f, "TTRX {limit} limit exceeded"),
            Self::InvalidUtf8 => f.write_str("invalid UTF-8 in TTRX string dictionary"),
            Self::InvalidNumber => f.write_str("invalid JSON number in TTRX payload"),
            Self::InvalidStringReference(index) => {
                write!(f, "invalid TTRX string reference {index}")
            }
            Self::InvalidShapeReference(index) => {
                write!(f, "invalid TTRX object-shape reference {index}")
            }
            Self::UnknownValueTag(tag) => write!(f, "unknown TTRX value tag {tag}"),
            Self::StatisticMismatch(statistic) => {
                write!(f, "TTRX {statistic} statistic does not match the payload")
            }
            Self::TrailingData => f.write_str("trailing data after the TTRX document"),
        }
    }
}

impl StdError for CodecError {}

#[derive(Clone, Copy)]
enum IntegerEncoding {
    Unsigned(u64),
    Signed(i64),
}

#[derive(Clone, Copy)]
struct ShapeOccurrence {
    count: u64,
    first_seen: u64,
}

struct Gathered {
    strings: Vec<String>,
    string_ids: BTreeMap<String, u64>,
    shapes: Vec<Vec<u64>>,
    shape_ids: BTreeMap<Vec<u64>, u64>,
    value_count: u64,
    max_depth: u32,
}

struct Gatherer {
    strings: Vec<String>,
    string_ids: BTreeMap<String, u64>,
    shape_occurrences: BTreeMap<Vec<u64>, ShapeOccurrence>,
    object_order: u64,
    value_count: u64,
    max_depth: u32,
    dictionary_bytes: u64,
    materialized_text_bytes: u64,
}

impl Gatherer {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            string_ids: BTreeMap::new(),
            shape_occurrences: BTreeMap::new(),
            object_order: 0,
            value_count: 0,
            max_depth: 0,
            dictionary_bytes: 0,
            materialized_text_bytes: 0,
        }
    }

    fn intern(&mut self, text: &str) -> Result<u64, CodecError> {
        if let Some(&id) = self.string_ids.get(text) {
            return Ok(id);
        }
        if u64::try_from(text.len()).map_err(|_| CodecError::LengthOverflow)? > MAX_STRING_BYTES {
            return Err(CodecError::LimitExceeded("string length"));
        }
        charge_budget(
            &mut self.dictionary_bytes,
            text.len(),
            MAX_DICTIONARY_BYTES,
            "dictionary byte count",
        )?;
        let id = u64::try_from(self.strings.len()).map_err(|_| CodecError::LengthOverflow)?;
        if id >= MAX_DICTIONARY_ENTRIES {
            return Err(CodecError::LimitExceeded("dictionary entry count"));
        }
        let owned = text.to_owned();
        self.strings.push(owned.clone());
        self.string_ids.insert(owned, id);
        Ok(id)
    }

    fn visit(&mut self, value: &JsonValue, depth: u32) -> Result<(), CodecError> {
        if depth > MAX_DEPTH {
            return Err(CodecError::LimitExceeded("nesting depth"));
        }
        self.value_count = self
            .value_count
            .checked_add(1)
            .ok_or(CodecError::LengthOverflow)?;
        if self.value_count > MAX_VALUE_COUNT {
            return Err(CodecError::LimitExceeded("value count"));
        }
        self.max_depth = self.max_depth.max(depth);

        match value {
            JsonValue::Null | JsonValue::Bool(_) => {}
            JsonValue::Number(number) => {
                if !is_valid_json_number(number) {
                    return Err(CodecError::InvalidNumber);
                }
                charge_budget(
                    &mut self.materialized_text_bytes,
                    number.len(),
                    MAX_MATERIALIZED_TEXT_BYTES,
                    "materialized text byte count",
                )?;
                if canonical_integer(number).is_none() {
                    self.intern(number)?;
                }
            }
            JsonValue::String(text) => {
                charge_budget(
                    &mut self.materialized_text_bytes,
                    text.len(),
                    MAX_MATERIALIZED_TEXT_BYTES,
                    "materialized text byte count",
                )?;
                self.intern(text)?;
            }
            JsonValue::Array(values) => {
                enforce_collection_len(values.len())?;
                let child_depth = depth.checked_add(1).ok_or(CodecError::LengthOverflow)?;
                for child in values {
                    self.visit(child, child_depth)?;
                }
            }
            JsonValue::Object(members) => {
                enforce_collection_len(members.len())?;
                let mut shape = Vec::with_capacity(members.len());
                for (key, _) in members {
                    charge_budget(
                        &mut self.materialized_text_bytes,
                        key.len(),
                        MAX_MATERIALIZED_TEXT_BYTES,
                        "materialized text byte count",
                    )?;
                    shape.push(self.intern(key)?);
                }
                let order = self.object_order;
                self.object_order = self
                    .object_order
                    .checked_add(1)
                    .ok_or(CodecError::LengthOverflow)?;
                let occurrence = self
                    .shape_occurrences
                    .entry(shape)
                    .or_insert(ShapeOccurrence {
                        count: 0,
                        first_seen: order,
                    });
                occurrence.count = occurrence
                    .count
                    .checked_add(1)
                    .ok_or(CodecError::LengthOverflow)?;

                let child_depth = depth.checked_add(1).ok_or(CodecError::LengthOverflow)?;
                for (_, child) in members {
                    self.visit(child, child_depth)?;
                }
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<Gathered, CodecError> {
        let mut repeated: Vec<(u64, Vec<u64>)> = self
            .shape_occurrences
            .into_iter()
            .filter_map(|(shape, occurrence)| {
                (occurrence.count >= 2).then_some((occurrence.first_seen, shape))
            })
            .collect();
        repeated.sort_by_key(|(first_seen, _)| *first_seen);
        if u64::try_from(repeated.len()).map_err(|_| CodecError::LengthOverflow)?
            > MAX_SHAPE_ENTRIES
        {
            return Err(CodecError::LimitExceeded("object-shape count"));
        }

        let shapes: Vec<Vec<u64>> = repeated.into_iter().map(|(_, shape)| shape).collect();
        let mut shape_ids = BTreeMap::new();
        for (index, shape) in shapes.iter().enumerate() {
            let index = u64::try_from(index).map_err(|_| CodecError::LengthOverflow)?;
            shape_ids.insert(shape.clone(), index);
        }

        Ok(Gathered {
            strings: self.strings,
            string_ids: self.string_ids,
            shapes,
            shape_ids,
            value_count: self.value_count,
            max_depth: self.max_depth,
        })
    }
}

/// Encode one parsed replay as a deterministic TTRX 1.0 source-semantic container.
///
/// # Errors
///
/// Returns an error if a programmatically constructed number is not valid JSON,
/// a document exceeds a resource limit, or a platform length conversion fails.
pub fn encode(
    value: &JsonValue,
    source_kind: SourceKind,
    original_json_len: u64,
) -> Result<Vec<u8>, CodecError> {
    if original_json_len > MAX_SOURCE_JSON_BYTES {
        return Err(CodecError::LimitExceeded("original JSON length"));
    }
    let mut gatherer = Gatherer::new();
    gatherer.visit(value, 1)?;
    let tables = gatherer.finish()?;

    let mut payload = Vec::new();
    for text in &tables.strings {
        write_len(text.len(), &mut payload)?;
        payload.extend_from_slice(text.as_bytes());
    }
    for shape in &tables.shapes {
        write_len(shape.len(), &mut payload)?;
        for &string_id in shape {
            varint::write_u64(string_id, &mut payload);
        }
    }
    encode_value(value, &tables, &mut payload, 1)?;

    let payload_len = u64::try_from(payload.len()).map_err(|_| CodecError::LengthOverflow)?;
    if payload_len > MAX_PAYLOAD_BYTES {
        return Err(CodecError::LimitExceeded("payload size"));
    }
    let payload_crc32c = crc32c::crc32c(&payload);
    let dictionary_entries =
        u64::try_from(tables.strings.len()).map_err(|_| CodecError::LengthOverflow)?;
    let shape_entries =
        u64::try_from(tables.shapes.len()).map_err(|_| CodecError::LengthOverflow)?;

    let mut output = Vec::with_capacity(
        HEADER_LEN
            .checked_add(payload.len())
            .ok_or(CodecError::LengthOverflow)?,
    );
    output.extend_from_slice(MAGIC);
    output.push(FORMAT_MAJOR);
    output.push(FORMAT_MINOR);
    output.push(FormatProfile::SourceSemantic.as_u8());
    output.push(source_kind.as_u8());
    push_u16(0, &mut output); // flags
    push_u16(HEADER_LEN_U16, &mut output);
    push_u32(0, &mut output); // reserved
    push_u64(original_json_len, &mut output);
    push_u64(payload_len, &mut output);
    push_u32(payload_crc32c, &mut output);
    push_u32(0, &mut output); // reserved
    push_u64(dictionary_entries, &mut output);
    push_u64(shape_entries, &mut output);
    push_u64(tables.value_count, &mut output);
    push_u32(tables.max_depth, &mut output);
    debug_assert_eq!(output.len(), HEADER_CRC_OFFSET);
    let header_crc32c = crc32c::crc32c(&output);
    push_u32(header_crc32c, &mut output);
    debug_assert_eq!(output.len(), HEADER_LEN);
    output.extend_from_slice(&payload);
    Ok(output)
}

fn encode_value(
    value: &JsonValue,
    gathered: &Gathered,
    output: &mut Vec<u8>,
    depth: u32,
) -> Result<(), CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::LimitExceeded("nesting depth"));
    }
    match value {
        JsonValue::Null => output.push(TAG_NULL),
        JsonValue::Bool(false) => output.push(TAG_FALSE),
        JsonValue::Bool(true) => output.push(TAG_TRUE),
        JsonValue::Number(number) => match canonical_integer(number) {
            Some(IntegerEncoding::Unsigned(value)) => {
                output.push(TAG_UNSIGNED_INTEGER);
                varint::write_u64(value, output);
            }
            Some(IntegerEncoding::Signed(value)) => {
                output.push(TAG_SIGNED_INTEGER);
                varint::write_i64(value, output);
            }
            None => {
                output.push(TAG_RAW_NUMBER);
                varint::write_u64(required_string_id(gathered, number)?, output);
            }
        },
        JsonValue::String(text) => {
            output.push(TAG_STRING);
            varint::write_u64(required_string_id(gathered, text)?, output);
        }
        JsonValue::Array(values) => {
            output.push(TAG_ARRAY);
            write_len(values.len(), output)?;
            let child_depth = depth.checked_add(1).ok_or(CodecError::LengthOverflow)?;
            for child in values {
                encode_value(child, gathered, output, child_depth)?;
            }
        }
        JsonValue::Object(members) => {
            let shape: Vec<u64> = members
                .iter()
                .map(|(key, _)| required_string_id(gathered, key))
                .collect::<Result<_, _>>()?;
            if let Some(&shape_id) = gathered.shape_ids.get(&shape) {
                output.push(TAG_OBJECT_SHAPE);
                varint::write_u64(shape_id, output);
            } else {
                output.push(TAG_OBJECT_INLINE);
                write_len(members.len(), output)?;
                for &key_id in &shape {
                    varint::write_u64(key_id, output);
                }
            }
            let child_depth = depth.checked_add(1).ok_or(CodecError::LengthOverflow)?;
            for (_, child) in members {
                encode_value(child, gathered, output, child_depth)?;
            }
        }
    }
    Ok(())
}

fn required_string_id(gathered: &Gathered, text: &str) -> Result<u64, CodecError> {
    gathered
        .string_ids
        .get(text)
        .copied()
        .ok_or(CodecError::InvalidHeader(
            "encoder dictionary is incomplete",
        ))
}

/// Decode and validate one complete TTRX 1.0 container.
///
/// # Errors
///
/// Returns an error for an unsupported or malformed header, checksum failure,
/// invalid payload reference or tag, a resource-limit violation, truncation, or
/// any bytes following the single declared document.
pub fn decode(container: &[u8]) -> Result<DecodedDocument, CodecError> {
    let (info, payload) = parse_container_header(container)?;
    let (dictionary, shapes, cursor) =
        decode_tables(payload, info.dictionary_entries, info.shape_entries)?;
    let mut decoder = ValueDecoder {
        input: payload,
        cursor,
        dictionary: &dictionary,
        shapes: &shapes,
        expected_values: info.value_count,
        seen_values: 0,
        observed_depth: 0,
        materialized_text_bytes: 0,
    };
    let value = decoder.read_value(1)?;
    if decoder.cursor != payload.len() {
        return Err(CodecError::TrailingData);
    }
    if decoder.seen_values != info.value_count {
        return Err(CodecError::StatisticMismatch("value count"));
    }
    if decoder.observed_depth != info.max_depth {
        return Err(CodecError::StatisticMismatch("maximum depth"));
    }

    Ok(DecodedDocument { value, info })
}

fn parse_container_header(container: &[u8]) -> Result<(ContainerInfo, &[u8]), CodecError> {
    if container.len() < HEADER_LEN {
        return Err(CodecError::UnexpectedEof);
    }
    if &container[..4] != MAGIC {
        return Err(CodecError::BadMagic);
    }
    let major = container[4];
    let minor = container[5];
    if major != FORMAT_MAJOR || minor != FORMAT_MINOR {
        return Err(CodecError::UnsupportedVersion { major, minor });
    }
    let profile = FormatProfile::from_u8(container[6])?;
    let source_kind = SourceKind::from_u8(container[7])?;
    if read_u16_at(container, 8)? != 0 {
        return Err(CodecError::InvalidHeader("nonzero flags"));
    }
    if read_u16_at(container, 10)? != HEADER_LEN_U16 {
        return Err(CodecError::InvalidHeader("unexpected header length"));
    }
    if read_u32_at(container, 12)? != 0 || read_u32_at(container, 36)? != 0 {
        return Err(CodecError::InvalidHeader("nonzero reserved field"));
    }

    let stored_header_crc = read_u32_at(container, HEADER_CRC_OFFSET)?;
    let actual_header_crc = crc32c::crc32c(&container[..HEADER_CRC_OFFSET]);
    if stored_header_crc != actual_header_crc {
        return Err(CodecError::HeaderChecksumMismatch {
            expected: stored_header_crc,
            actual: actual_header_crc,
        });
    }

    let original_json_len = read_u64_at(container, 16)?;
    let payload_len = read_u64_at(container, 24)?;
    let payload_crc32c = read_u32_at(container, 32)?;
    let dictionary_entries = read_u64_at(container, 40)?;
    let shape_entries = read_u64_at(container, 48)?;
    let value_count = read_u64_at(container, 56)?;
    let max_depth = read_u32_at(container, 64)?;

    enforce_header_limits(
        original_json_len,
        payload_len,
        dictionary_entries,
        shape_entries,
        value_count,
        max_depth,
    )?;

    let payload_len_usize = usize::try_from(payload_len).map_err(|_| CodecError::LengthOverflow)?;
    let expected_len = HEADER_LEN
        .checked_add(payload_len_usize)
        .ok_or(CodecError::LengthOverflow)?;
    if container.len() < expected_len {
        return Err(CodecError::UnexpectedEof);
    }
    if container.len() > expected_len {
        return Err(CodecError::TrailingData);
    }
    let payload = &container[HEADER_LEN..expected_len];
    let actual_payload_crc = crc32c::crc32c(payload);
    if payload_crc32c != actual_payload_crc {
        return Err(CodecError::PayloadChecksumMismatch {
            expected: payload_crc32c,
            actual: actual_payload_crc,
        });
    }

    Ok((
        ContainerInfo {
            format_major: major,
            format_minor: minor,
            profile,
            source_kind,
            original_json_len,
            payload_len,
            payload_crc32c,
            dictionary_entries,
            shape_entries,
            value_count,
            max_depth,
        },
        payload,
    ))
}

type DecodedTables = (Vec<String>, Vec<Vec<u64>>, usize);

fn decode_tables(
    payload: &[u8],
    dictionary_entries: u64,
    shape_entries: u64,
) -> Result<DecodedTables, CodecError> {
    let minimum_table_and_root_bytes = dictionary_entries
        .checked_add(shape_entries)
        .and_then(|value| value.checked_add(1))
        .ok_or(CodecError::LengthOverflow)?;
    if minimum_table_and_root_bytes
        > u64::try_from(payload.len()).map_err(|_| CodecError::LengthOverflow)?
    {
        return Err(CodecError::InvalidHeader(
            "table counts exceed payload capacity",
        ));
    }

    let mut cursor = 0;
    let dictionary_capacity =
        usize::try_from(dictionary_entries).map_err(|_| CodecError::LengthOverflow)?;
    let mut dictionary = Vec::with_capacity(dictionary_capacity);
    let mut dictionary_bytes = 0_u64;
    for _ in 0..dictionary_entries {
        let len = read_usize_varint(payload, &mut cursor)?;
        if u64::try_from(len).map_err(|_| CodecError::LengthOverflow)? > MAX_STRING_BYTES {
            return Err(CodecError::LimitExceeded("string length"));
        }
        charge_budget(
            &mut dictionary_bytes,
            len,
            MAX_DICTIONARY_BYTES,
            "dictionary byte count",
        )?;
        let end = cursor.checked_add(len).ok_or(CodecError::LengthOverflow)?;
        let bytes = payload.get(cursor..end).ok_or(CodecError::UnexpectedEof)?;
        let text = std::str::from_utf8(bytes).map_err(|_| CodecError::InvalidUtf8)?;
        dictionary.push(text.to_owned());
        cursor = end;
    }

    let shapes_capacity = usize::try_from(shape_entries).map_err(|_| CodecError::LengthOverflow)?;
    let mut shapes = Vec::with_capacity(shapes_capacity);
    let mut shape_key_references = 0_u64;
    for _ in 0..shape_entries {
        let member_count = read_usize_varint(payload, &mut cursor)?;
        if u64::try_from(member_count).map_err(|_| CodecError::LengthOverflow)?
            > MAX_COLLECTION_ITEMS
        {
            return Err(CodecError::LimitExceeded("object member count"));
        }
        charge_budget(
            &mut shape_key_references,
            member_count,
            MAX_SHAPE_KEY_REFERENCES,
            "shape key reference count",
        )?;
        if member_count > payload.len().saturating_sub(cursor) {
            return Err(CodecError::UnexpectedEof);
        }
        let mut shape = Vec::with_capacity(member_count);
        for _ in 0..member_count {
            let key_id = read_u64_varint(payload, &mut cursor)?;
            checked_string(&dictionary, key_id)?;
            shape.push(key_id);
        }
        shapes.push(shape);
    }

    Ok((dictionary, shapes, cursor))
}

struct ValueDecoder<'a> {
    input: &'a [u8],
    cursor: usize,
    dictionary: &'a [String],
    shapes: &'a [Vec<u64>],
    expected_values: u64,
    seen_values: u64,
    observed_depth: u32,
    materialized_text_bytes: u64,
}

impl ValueDecoder<'_> {
    fn read_value(&mut self, depth: u32) -> Result<JsonValue, CodecError> {
        if depth > MAX_DEPTH {
            return Err(CodecError::LimitExceeded("nesting depth"));
        }
        self.seen_values = self
            .seen_values
            .checked_add(1)
            .ok_or(CodecError::LengthOverflow)?;
        if self.seen_values > self.expected_values || self.seen_values > MAX_VALUE_COUNT {
            return Err(CodecError::LimitExceeded("value count"));
        }
        self.observed_depth = self.observed_depth.max(depth);
        let tag = *self
            .input
            .get(self.cursor)
            .ok_or(CodecError::UnexpectedEof)?;
        self.cursor += 1;
        match tag {
            TAG_NULL => Ok(JsonValue::Null),
            TAG_FALSE => Ok(JsonValue::Bool(false)),
            TAG_TRUE => Ok(JsonValue::Bool(true)),
            TAG_UNSIGNED_INTEGER => {
                let value = read_u64_varint(self.input, &mut self.cursor)?;
                let number = value.to_string();
                self.charge_materialized_text(number.len())?;
                Ok(JsonValue::Number(number))
            }
            TAG_SIGNED_INTEGER => {
                let value = read_i64_varint(self.input, &mut self.cursor)?;
                if value >= 0 {
                    return Err(CodecError::InvalidNumber);
                }
                let number = value.to_string();
                self.charge_materialized_text(number.len())?;
                Ok(JsonValue::Number(number))
            }
            TAG_RAW_NUMBER => {
                let index = read_u64_varint(self.input, &mut self.cursor)?;
                let number = checked_string(self.dictionary, index)?;
                if !is_valid_json_number(number) || canonical_integer(number).is_some() {
                    return Err(CodecError::InvalidNumber);
                }
                Ok(JsonValue::Number(
                    self.materialize_dictionary_string(index)?,
                ))
            }
            TAG_STRING => {
                let index = read_u64_varint(self.input, &mut self.cursor)?;
                Ok(JsonValue::String(
                    self.materialize_dictionary_string(index)?,
                ))
            }
            TAG_ARRAY => {
                let len = read_usize_varint(self.input, &mut self.cursor)?;
                self.ensure_children_fit(len)?;
                let mut values = Vec::with_capacity(len);
                let child_depth = depth.checked_add(1).ok_or(CodecError::LengthOverflow)?;
                for _ in 0..len {
                    values.push(self.read_value(child_depth)?);
                }
                Ok(JsonValue::Array(values))
            }
            TAG_OBJECT_INLINE => {
                let len = read_usize_varint(self.input, &mut self.cursor)?;
                self.ensure_children_fit(len)?;
                let mut keys = Vec::with_capacity(len);
                for _ in 0..len {
                    let index = read_u64_varint(self.input, &mut self.cursor)?;
                    keys.push(self.materialize_dictionary_string(index)?);
                }
                self.read_object_values(keys, depth)
            }
            TAG_OBJECT_SHAPE => {
                let index = read_u64_varint(self.input, &mut self.cursor)?;
                let shape_index =
                    usize::try_from(index).map_err(|_| CodecError::InvalidShapeReference(index))?;
                let shape = self
                    .shapes
                    .get(shape_index)
                    .ok_or(CodecError::InvalidShapeReference(index))?;
                self.ensure_children_fit(shape.len())?;
                let mut keys = Vec::with_capacity(shape.len());
                for &key_id in shape {
                    keys.push(self.materialize_dictionary_string(key_id)?);
                }
                self.read_object_values(keys, depth)
            }
            other => Err(CodecError::UnknownValueTag(other)),
        }
    }

    fn ensure_children_fit(&self, len: usize) -> Result<(), CodecError> {
        let len = u64::try_from(len).map_err(|_| CodecError::LengthOverflow)?;
        if len > MAX_COLLECTION_ITEMS {
            return Err(CodecError::LimitExceeded("collection item count"));
        }
        let remaining = self
            .expected_values
            .checked_sub(self.seen_values)
            .ok_or(CodecError::StatisticMismatch("value count"))?;
        if len > remaining {
            return Err(CodecError::StatisticMismatch("value count"));
        }
        Ok(())
    }

    fn charge_materialized_text(&mut self, bytes: usize) -> Result<(), CodecError> {
        charge_budget(
            &mut self.materialized_text_bytes,
            bytes,
            MAX_MATERIALIZED_TEXT_BYTES,
            "materialized text byte count",
        )
    }

    fn materialize_dictionary_string(&mut self, index: u64) -> Result<String, CodecError> {
        let byte_len = checked_string(self.dictionary, index)?.len();
        self.charge_materialized_text(byte_len)?;
        Ok(checked_string(self.dictionary, index)?.to_owned())
    }

    fn read_object_values(
        &mut self,
        keys: Vec<String>,
        depth: u32,
    ) -> Result<JsonValue, CodecError> {
        let child_depth = depth.checked_add(1).ok_or(CodecError::LengthOverflow)?;
        let mut members = Vec::with_capacity(keys.len());
        for key in keys {
            let value = self.read_value(child_depth)?;
            members.push((key, value));
        }
        Ok(JsonValue::Object(members))
    }
}

fn enforce_header_limits(
    original_json_len: u64,
    payload_len: u64,
    dictionary_entries: u64,
    shape_entries: u64,
    value_count: u64,
    max_depth: u32,
) -> Result<(), CodecError> {
    if original_json_len > MAX_SOURCE_JSON_BYTES {
        return Err(CodecError::LimitExceeded("original JSON length"));
    }
    if payload_len > MAX_PAYLOAD_BYTES {
        return Err(CodecError::LimitExceeded("payload size"));
    }
    if dictionary_entries > MAX_DICTIONARY_ENTRIES {
        return Err(CodecError::LimitExceeded("dictionary entry count"));
    }
    if shape_entries > MAX_SHAPE_ENTRIES {
        return Err(CodecError::LimitExceeded("object-shape count"));
    }
    if value_count == 0 || value_count > MAX_VALUE_COUNT {
        return Err(CodecError::LimitExceeded("value count"));
    }
    if max_depth == 0 || max_depth > MAX_DEPTH {
        return Err(CodecError::LimitExceeded("nesting depth"));
    }
    Ok(())
}

fn enforce_collection_len(len: usize) -> Result<(), CodecError> {
    if u64::try_from(len).map_err(|_| CodecError::LengthOverflow)? > MAX_COLLECTION_ITEMS {
        return Err(CodecError::LimitExceeded("collection item count"));
    }
    Ok(())
}

fn charge_budget(
    used: &mut u64,
    amount: usize,
    limit: u64,
    name: &'static str,
) -> Result<(), CodecError> {
    let amount = u64::try_from(amount).map_err(|_| CodecError::LengthOverflow)?;
    *used = used.checked_add(amount).ok_or(CodecError::LengthOverflow)?;
    if *used > limit {
        return Err(CodecError::LimitExceeded(name));
    }
    Ok(())
}

fn canonical_integer(number: &str) -> Option<IntegerEncoding> {
    if number == "0" {
        return Some(IntegerEncoding::Unsigned(0));
    }
    if let Some(digits) = number.strip_prefix('-') {
        if digits.is_empty()
            || digits.starts_with('0')
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        return number.parse::<i64>().ok().map(IntegerEncoding::Signed);
    }
    if number.starts_with('0') || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    number.parse::<u64>().ok().map(IntegerEncoding::Unsigned)
}

fn is_valid_json_number(number: &str) -> bool {
    let bytes = number.as_bytes();
    let mut index = 0;
    if bytes.get(index) == Some(&b'-') {
        index += 1;
    }
    match bytes.get(index) {
        Some(b'0') => index += 1,
        Some(b'1'..=b'9') => {
            index += 1;
            while matches!(bytes.get(index), Some(b'0'..=b'9')) {
                index += 1;
            }
        }
        _ => return false,
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
        if index == fraction_start {
            return false;
        }
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
        if index == exponent_start {
            return false;
        }
    }
    index == bytes.len()
}

fn checked_string(dictionary: &[String], index: u64) -> Result<&str, CodecError> {
    let index_usize =
        usize::try_from(index).map_err(|_| CodecError::InvalidStringReference(index))?;
    dictionary
        .get(index_usize)
        .map(String::as_str)
        .ok_or(CodecError::InvalidStringReference(index))
}

fn write_len(len: usize, output: &mut Vec<u8>) -> Result<(), CodecError> {
    let len = u64::try_from(len).map_err(|_| CodecError::LengthOverflow)?;
    varint::write_u64(len, output);
    Ok(())
}

fn read_u64_varint(input: &[u8], cursor: &mut usize) -> Result<u64, CodecError> {
    let start = *cursor;
    let value = varint::read_u64(input, cursor)?;
    if *cursor - start != varint_len(value) {
        return Err(CodecError::NonCanonicalVarint);
    }
    Ok(value)
}

fn read_i64_varint(input: &[u8], cursor: &mut usize) -> Result<i64, CodecError> {
    let encoded = read_u64_varint(input, cursor)?;
    let magnitude = i64::try_from(encoded >> 1).map_err(|_| CodecError::InvalidVarint)?;
    if encoded & 1 == 0 {
        Ok(magnitude)
    } else {
        Ok(-magnitude - 1)
    }
}

fn read_usize_varint(input: &[u8], cursor: &mut usize) -> Result<usize, CodecError> {
    let value = read_u64_varint(input, cursor)?;
    usize::try_from(value).map_err(|_| CodecError::LengthOverflow)
}

fn varint_len(mut value: u64) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        value >>= 7;
        len += 1;
    }
    len
}

fn push_u16(value: u16, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(value: u32, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(value: u64, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn read_u16_at(input: &[u8], offset: usize) -> Result<u16, CodecError> {
    let bytes: [u8; 2] = input
        .get(offset..offset + 2)
        .ok_or(CodecError::UnexpectedEof)?
        .try_into()
        .map_err(|_| CodecError::UnexpectedEof)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32_at(input: &[u8], offset: usize) -> Result<u32, CodecError> {
    let bytes: [u8; 4] = input
        .get(offset..offset + 4)
        .ok_or(CodecError::UnexpectedEof)?
        .try_into()
        .map_err(|_| CodecError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64_at(input: &[u8], offset: usize) -> Result<u64, CodecError> {
    let bytes: [u8; 8] = input
        .get(offset..offset + 8)
        .ok_or(CodecError::UnexpectedEof)?
        .try_into()
        .map_err(|_| CodecError::UnexpectedEof)?;
    Ok(u64::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::{
        CodecError, HEADER_CRC_OFFSET, HEADER_LEN, MAX_DICTIONARY_BYTES,
        MAX_MATERIALIZED_TEXT_BYTES, MAX_SHAPE_KEY_REFERENCES, MAX_SOURCE_JSON_BYTES, SourceKind,
        TAG_STRING, ValueDecoder, charge_budget, decode, encode,
    };
    use crate::crc32c::crc32c;
    use crate::json::JsonValue;

    fn object(members: Vec<(&str, JsonValue)>) -> JsonValue {
        JsonValue::Object(
            members
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    }

    fn rewrite_checksums(container: &mut [u8]) {
        let payload_len = u64::try_from(container.len() - HEADER_LEN).unwrap();
        container[24..32].copy_from_slice(&payload_len.to_le_bytes());
        let payload_crc = crc32c(&container[HEADER_LEN..]);
        container[32..36].copy_from_slice(&payload_crc.to_le_bytes());
        let header_crc = crc32c(&container[..HEADER_CRC_OFFSET]);
        container[HEADER_CRC_OFFSET..HEADER_LEN].copy_from_slice(&header_crc.to_le_bytes());
    }

    #[test]
    fn round_trip_preserves_the_json_data_model() {
        let repeated_a = object(vec![
            ("frame", JsonValue::Number("18446744073709551615".into())),
            ("subframe", JsonValue::Number("1.00".into())),
        ]);
        let repeated_b = object(vec![
            ("frame", JsonValue::Number("-9223372036854775808".into())),
            ("subframe", JsonValue::Number("-0".into())),
        ]);
        let value = object(vec![
            ("events", JsonValue::Array(vec![repeated_a, repeated_b])),
            ("title", JsonValue::String("테트리스".into())),
            ("truth", JsonValue::Bool(true)),
            ("empty", JsonValue::Null),
        ]);

        let encoded = encode(&value, SourceKind::Ttr, 1234).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.value, value);
        assert_eq!(decoded.info.source_kind, SourceKind::Ttr);
        assert_eq!(decoded.info.original_json_len, 1234);
        assert_eq!(decoded.info.shape_entries, 1);
        assert_eq!(decoded.info.value_count, 11);
        assert_eq!(decoded.info.max_depth, 4);
    }

    #[test]
    fn duplicate_object_keys_and_order_survive() {
        let value = object(vec![
            ("same", JsonValue::Number("1".into())),
            ("same", JsonValue::Number("2".into())),
            ("before", JsonValue::String("after?".into())),
        ]);
        let decoded = decode(&encode(&value, SourceKind::Unknown, 0).unwrap()).unwrap();
        assert_eq!(decoded.value, value);
    }

    #[test]
    fn output_is_deterministic() {
        let value = object(vec![
            ("a", JsonValue::String("shared".into())),
            ("b", JsonValue::String("shared".into())),
        ]);
        let first = encode(&value, SourceKind::Ttrm, 99).unwrap();
        let second = encode(&value, SourceKind::Ttrm, 99).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn payload_corruption_is_rejected() {
        let mut encoded = encode(&JsonValue::Null, SourceKind::Ttr, 4).unwrap();
        *encoded.last_mut().unwrap() ^= 1;
        assert!(matches!(
            decode(&encoded),
            Err(CodecError::PayloadChecksumMismatch { .. })
        ));
    }

    #[test]
    fn truncation_is_rejected() {
        let mut encoded = encode(&JsonValue::String("value".into()), SourceKind::Ttr, 7).unwrap();
        encoded.pop();
        assert_eq!(decode(&encoded), Err(CodecError::UnexpectedEof));
    }

    #[test]
    fn unknown_version_is_rejected() {
        let mut encoded = encode(&JsonValue::Null, SourceKind::Ttr, 4).unwrap();
        encoded[4] = 99;
        assert_eq!(
            decode(&encoded),
            Err(CodecError::UnsupportedVersion {
                major: 99,
                minor: 0
            })
        );
    }

    #[test]
    fn unknown_value_tag_is_rejected_after_checksum_validation() {
        let mut encoded = encode(&JsonValue::Null, SourceKind::Ttr, 4).unwrap();
        encoded[HEADER_LEN] = 0xff;
        rewrite_checksums(&mut encoded);
        assert_eq!(decode(&encoded), Err(CodecError::UnknownValueTag(0xff)));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut encoded = encode(&JsonValue::Null, SourceKind::Ttr, 4).unwrap();
        encoded.push(0);
        assert_eq!(decode(&encoded), Err(CodecError::TrailingData));
    }

    #[test]
    fn invalid_number_from_a_programmatic_caller_is_rejected() {
        assert_eq!(
            encode(&JsonValue::Number("01".into()), SourceKind::Unknown, 2),
            Err(CodecError::InvalidNumber)
        );
    }

    #[test]
    fn original_source_length_limit_is_enforced_on_encode_and_decode() {
        assert_eq!(
            encode(
                &JsonValue::Null,
                SourceKind::Unknown,
                MAX_SOURCE_JSON_BYTES + 1
            ),
            Err(CodecError::LimitExceeded("original JSON length"))
        );

        let mut encoded = encode(&JsonValue::Null, SourceKind::Unknown, 4).unwrap();
        encoded[16..24].copy_from_slice(&(MAX_SOURCE_JSON_BYTES + 1).to_le_bytes());
        rewrite_checksums(&mut encoded);
        assert_eq!(
            decode(&encoded),
            Err(CodecError::LimitExceeded("original JSON length"))
        );
    }

    #[test]
    fn impossible_table_counts_are_rejected_before_table_allocation() {
        let mut encoded = encode(&JsonValue::Null, SourceKind::Unknown, 4).unwrap();
        encoded[40..48].copy_from_slice(&1_u64.to_le_bytes());
        rewrite_checksums(&mut encoded);
        assert_eq!(
            decode(&encoded),
            Err(CodecError::InvalidHeader(
                "table counts exceed payload capacity"
            ))
        );
    }

    #[test]
    fn noncanonical_payload_varint_is_rejected() {
        let mut encoded =
            encode(&JsonValue::String(String::new()), SourceKind::Unknown, 2).unwrap();
        // Payload was [dictionary length 0, string tag, dictionary ID 0].
        // Insert a continuation byte to spell ID zero as overlong [0x80, 0x00].
        encoded.insert(HEADER_LEN + 2, 0x80);
        rewrite_checksums(&mut encoded);
        assert_eq!(decode(&encoded), Err(CodecError::NonCanonicalVarint));
    }

    #[test]
    fn repeated_dictionary_materialization_obeys_the_global_budget() {
        let dictionary = vec!["abc".to_owned()];
        let shapes = Vec::new();
        let input = [TAG_STRING, 0];
        let mut decoder = ValueDecoder {
            input: &input,
            cursor: 0,
            dictionary: &dictionary,
            shapes: &shapes,
            expected_values: 1,
            seen_values: 0,
            observed_depth: 0,
            materialized_text_bytes: MAX_MATERIALIZED_TEXT_BYTES - 2,
        };
        assert_eq!(
            decoder.read_value(1),
            Err(CodecError::LimitExceeded("materialized text byte count"))
        );
    }

    #[test]
    fn aggregate_dictionary_and_shape_budgets_reject_excess() {
        let mut dictionary_bytes = MAX_DICTIONARY_BYTES;
        assert_eq!(
            charge_budget(
                &mut dictionary_bytes,
                1,
                MAX_DICTIONARY_BYTES,
                "dictionary byte count"
            ),
            Err(CodecError::LimitExceeded("dictionary byte count"))
        );

        let mut shape_references = MAX_SHAPE_KEY_REFERENCES;
        assert_eq!(
            charge_budget(
                &mut shape_references,
                1,
                MAX_SHAPE_KEY_REFERENCES,
                "shape key reference count"
            ),
            Err(CodecError::LimitExceeded("shape key reference count"))
        );
    }
}
