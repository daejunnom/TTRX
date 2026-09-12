//! Minimal, lossless JSON data model for replay source documents.
//!
//! Numbers retain their source lexeme, and objects retain both member order and
//! duplicate keys. This is intentional: the TTRX source-semantic round trip
//! must not silently normalize data that JSON permits.

use std::error::Error as StdError;
use std::fmt;
use std::str;

const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_STRING_BYTES: usize = 16 * 1024 * 1024;
const MAX_COLLECTION_ITEMS: usize = 1_000_000;
const MAX_TOTAL_VALUES: usize = 2_000_000;
const MAX_DEPTH: usize = 512;
pub(crate) const MAX_JSON_BYTES: usize = MAX_INPUT_BYTES;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonError {
    offset: usize,
    kind: JsonErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum JsonErrorKind {
    InputTooLarge,
    InvalidUtf8,
    UnexpectedEnd,
    UnexpectedToken,
    ExpectedObjectKey,
    ExpectedColon,
    ExpectedCommaOrEnd,
    InvalidEscape,
    InvalidUnicodeEscape,
    UnpairedHighSurrogate,
    UnpairedLowSurrogate,
    UnescapedControlCharacter,
    InvalidNumber,
    TrailingCharacters,
    NestingTooDeep,
    CollectionTooLarge,
    StringTooLong,
    TooManyValues,
}

impl JsonError {
    fn new(offset: usize, kind: JsonErrorKind) -> Self {
        Self { offset, kind }
    }

    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            JsonErrorKind::InputTooLarge => "input exceeds the JSON byte limit",
            JsonErrorKind::InvalidUtf8 => "input is not valid UTF-8",
            JsonErrorKind::UnexpectedEnd => "unexpected end of input",
            JsonErrorKind::UnexpectedToken => "unexpected token",
            JsonErrorKind::ExpectedObjectKey => "expected an object member name",
            JsonErrorKind::ExpectedColon => "expected ':' after object member name",
            JsonErrorKind::ExpectedCommaOrEnd => "expected ',' or the end of the collection",
            JsonErrorKind::InvalidEscape => "invalid string escape",
            JsonErrorKind::InvalidUnicodeEscape => "invalid Unicode escape",
            JsonErrorKind::UnpairedHighSurrogate => "unpaired high surrogate",
            JsonErrorKind::UnpairedLowSurrogate => "unpaired low surrogate",
            JsonErrorKind::UnescapedControlCharacter => "unescaped control character in string",
            JsonErrorKind::InvalidNumber => "invalid JSON number",
            JsonErrorKind::TrailingCharacters => "trailing characters after the JSON value",
            JsonErrorKind::NestingTooDeep => "JSON nesting exceeds 512 collections",
            JsonErrorKind::CollectionTooLarge => "JSON collection has too many items",
            JsonErrorKind::StringTooLong => "decoded JSON string exceeds the byte limit",
            JsonErrorKind::TooManyValues => "JSON document contains too many values",
        };
        write!(f, "{message} at byte {}", self.offset)
    }
}

impl StdError for JsonError {}

#[derive(Clone, Copy)]
struct Limits {
    input_bytes: usize,
    string_bytes: usize,
    collection_items: usize,
    total_values: usize,
    depth: usize,
}

const DEFAULT_LIMITS: Limits = Limits {
    input_bytes: MAX_INPUT_BYTES,
    string_bytes: MAX_STRING_BYTES,
    collection_items: MAX_COLLECTION_ITEMS,
    total_values: MAX_TOTAL_VALUES,
    depth: MAX_DEPTH,
};

/// Parse one strict JSON document.
///
/// The parser accepts at most 64 MiB of input, 16 MiB per decoded string,
/// 1,000,000 members per collection, 2,000,000 values in total, and 512
/// nested arrays or objects.
/// Parse one strict UTF-8 JSON document while preserving number lexemes,
/// object-member order, and duplicate keys.
///
/// # Errors
///
/// Returns a byte-offset error for malformed UTF-8 or JSON and an explicit
/// resource-limit error for an oversized or excessively nested document.
pub fn parse(input: &[u8]) -> Result<JsonValue, JsonError> {
    parse_with_limits(input, DEFAULT_LIMITS)
}

fn parse_with_limits(input: &[u8], limits: Limits) -> Result<JsonValue, JsonError> {
    if input.len() > limits.input_bytes {
        return Err(JsonError::new(0, JsonErrorKind::InputTooLarge));
    }

    if let Err(error) = str::from_utf8(input) {
        return Err(JsonError::new(
            error.valid_up_to(),
            JsonErrorKind::InvalidUtf8,
        ));
    }

    Parser {
        input,
        cursor: 0,
        total_values: 0,
        limits,
    }
    .parse_document()
}

struct Parser<'a> {
    input: &'a [u8],
    cursor: usize,
    total_values: usize,
    limits: Limits,
}

impl Parser<'_> {
    fn parse_document(mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        let value = self.parse_value(0)?;
        self.skip_whitespace();
        if self.cursor != self.input.len() {
            return Err(self.error(JsonErrorKind::TrailingCharacters));
        }
        Ok(value)
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.count_value()?;
        match self
            .peek()
            .ok_or_else(|| self.error(JsonErrorKind::UnexpectedEnd))?
        {
            b'n' => {
                self.consume_literal(b"null")?;
                Ok(JsonValue::Null)
            }
            b't' => {
                self.consume_literal(b"true")?;
                Ok(JsonValue::Bool(true))
            }
            b'f' => {
                self.consume_literal(b"false")?;
                Ok(JsonValue::Bool(false))
            }
            b'"' => self.parse_string().map(JsonValue::String),
            b'[' => self.parse_array(depth),
            b'{' => self.parse_object(depth),
            b'-' | b'0'..=b'9' => self.parse_number().map(JsonValue::Number),
            _ => Err(self.error(JsonErrorKind::UnexpectedToken)),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.check_depth(depth)?;
        self.cursor += 1;
        self.skip_whitespace();

        let mut values = Vec::new();
        if self.take_if(b']') {
            return Ok(JsonValue::Array(values));
        }

        loop {
            if values.len() >= self.limits.collection_items {
                return Err(self.error(JsonErrorKind::CollectionTooLarge));
            }
            values.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();
            if self.take_if(b']') {
                return Ok(JsonValue::Array(values));
            }
            if !self.take_if(b',') {
                return Err(self.error(JsonErrorKind::ExpectedCommaOrEnd));
            }
            self.skip_whitespace();
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.check_depth(depth)?;
        self.cursor += 1;
        self.skip_whitespace();

        let mut members = Vec::new();
        if self.take_if(b'}') {
            return Ok(JsonValue::Object(members));
        }

        loop {
            if members.len() >= self.limits.collection_items {
                return Err(self.error(JsonErrorKind::CollectionTooLarge));
            }
            if self.peek() != Some(b'"') {
                return Err(self.error(JsonErrorKind::ExpectedObjectKey));
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if !self.take_if(b':') {
                return Err(self.error(JsonErrorKind::ExpectedColon));
            }
            self.skip_whitespace();
            let value = self.parse_value(depth + 1)?;
            members.push((key, value));
            self.skip_whitespace();
            if self.take_if(b'}') {
                return Ok(JsonValue::Object(members));
            }
            if !self.take_if(b',') {
                return Err(self.error(JsonErrorKind::ExpectedCommaOrEnd));
            }
            self.skip_whitespace();
        }
    }

    fn parse_string(&mut self) -> Result<String, JsonError> {
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.cursor += 1;
        let mut output = String::new();

        loop {
            let byte = self
                .peek()
                .ok_or_else(|| self.error(JsonErrorKind::UnexpectedEnd))?;
            match byte {
                b'"' => {
                    self.cursor += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.cursor += 1;
                    self.parse_escape(&mut output)?;
                }
                0x00..=0x1f => {
                    return Err(self.error(JsonErrorKind::UnescapedControlCharacter));
                }
                0x20..=0x7f => {
                    self.push_char(&mut output, char::from(byte))?;
                    self.cursor += 1;
                }
                _ => {
                    // The complete input was validated before parsing, so slicing at
                    // a non-ASCII byte here always starts a valid UTF-8 character.
                    let tail = str::from_utf8(&self.input[self.cursor..])
                        .expect("input was validated as UTF-8");
                    let character = tail.chars().next().expect("tail is non-empty");
                    self.push_char(&mut output, character)?;
                    self.cursor += character.len_utf8();
                }
            }
        }
    }

    fn parse_escape(&mut self, output: &mut String) -> Result<(), JsonError> {
        let escape_offset = self.cursor.saturating_sub(1);
        let escaped = self
            .peek()
            .ok_or_else(|| self.error(JsonErrorKind::UnexpectedEnd))?;
        self.cursor += 1;
        match escaped {
            b'"' => self.push_char(output, '"'),
            b'\\' => self.push_char(output, '\\'),
            b'/' => self.push_char(output, '/'),
            b'b' => self.push_char(output, '\u{0008}'),
            b'f' => self.push_char(output, '\u{000c}'),
            b'n' => self.push_char(output, '\n'),
            b'r' => self.push_char(output, '\r'),
            b't' => self.push_char(output, '\t'),
            b'u' => self.parse_unicode_escape(output, escape_offset),
            _ => Err(JsonError::new(escape_offset, JsonErrorKind::InvalidEscape)),
        }
    }

    fn parse_unicode_escape(
        &mut self,
        output: &mut String,
        escape_offset: usize,
    ) -> Result<(), JsonError> {
        let first = self.parse_hex_quad()?;
        match first {
            0xd800..=0xdbff => {
                if self.input.get(self.cursor..self.cursor.saturating_add(2)) != Some(b"\\u") {
                    return Err(JsonError::new(
                        escape_offset,
                        JsonErrorKind::UnpairedHighSurrogate,
                    ));
                }
                self.cursor += 2;
                let second_offset = self.cursor.saturating_sub(2);
                let second = self.parse_hex_quad()?;
                if !(0xdc00..=0xdfff).contains(&second) {
                    return Err(JsonError::new(
                        second_offset,
                        JsonErrorKind::UnpairedHighSurrogate,
                    ));
                }
                let scalar =
                    0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00);
                let character = char::from_u32(scalar).expect("surrogate pair is a Unicode scalar");
                self.push_char(output, character)
            }
            0xdc00..=0xdfff => Err(JsonError::new(
                escape_offset,
                JsonErrorKind::UnpairedLowSurrogate,
            )),
            _ => {
                let character = char::from_u32(u32::from(first))
                    .expect("non-surrogate u16 is a Unicode scalar");
                self.push_char(output, character)
            }
        }
    }

    fn parse_hex_quad(&mut self) -> Result<u16, JsonError> {
        let start = self.cursor;
        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self
                .peek()
                .ok_or_else(|| JsonError::new(start, JsonErrorKind::InvalidUnicodeEscape))?;
            let digit = hex_digit(byte)
                .ok_or_else(|| JsonError::new(self.cursor, JsonErrorKind::InvalidUnicodeEscape))?;
            value = (value << 4) | u16::from(digit);
            self.cursor += 1;
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<String, JsonError> {
        let start = self.cursor;
        self.take_if(b'-');

        match self.peek() {
            Some(b'0') => {
                self.cursor += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(JsonError::new(start, JsonErrorKind::InvalidNumber));
                }
            }
            Some(b'1'..=b'9') => {
                self.cursor += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.cursor += 1;
                }
            }
            _ => return Err(JsonError::new(start, JsonErrorKind::InvalidNumber)),
        }

        if self.take_if(b'.') {
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonError::new(start, JsonErrorKind::InvalidNumber));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.cursor += 1;
            }
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.cursor += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.cursor += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonError::new(start, JsonErrorKind::InvalidNumber));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.cursor += 1;
            }
        }

        let number = str::from_utf8(&self.input[start..self.cursor])
            .expect("JSON number bytes are ASCII")
            .to_owned();
        Ok(number)
    }

    fn consume_literal(&mut self, literal: &[u8]) -> Result<(), JsonError> {
        if self
            .input
            .get(self.cursor..self.cursor.saturating_add(literal.len()))
            == Some(literal)
        {
            self.cursor += literal.len();
            Ok(())
        } else {
            Err(self.error(JsonErrorKind::UnexpectedToken))
        }
    }

    fn count_value(&mut self) -> Result<(), JsonError> {
        if self.total_values >= self.limits.total_values {
            return Err(self.error(JsonErrorKind::TooManyValues));
        }
        self.total_values += 1;
        Ok(())
    }

    fn check_depth(&self, depth: usize) -> Result<(), JsonError> {
        if depth >= self.limits.depth {
            Err(self.error(JsonErrorKind::NestingTooDeep))
        } else {
            Ok(())
        }
    }

    fn push_char(&self, output: &mut String, character: char) -> Result<(), JsonError> {
        if output.len().saturating_add(character.len_utf8()) > self.limits.string_bytes {
            return Err(self.error(JsonErrorKind::StringTooLong));
        }
        output.push(character);
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.cursor += 1;
        }
    }

    fn take_if(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.cursor).copied()
    }

    fn error(&self, kind: JsonErrorKind) -> JsonError {
        JsonError::new(self.cursor, kind)
    }
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Serialize a JSON value as compact UTF-8 JSON.
///
/// Object member order, duplicate keys, and number lexemes are emitted exactly
/// as represented by [`JsonValue`]. String spelling is canonicalized.
#[must_use]
pub(crate) fn to_vec(value: &JsonValue) -> Vec<u8> {
    enum Frame<'a> {
        Array(&'a [JsonValue], usize),
        Object(&'a [(String, JsonValue)], usize),
    }

    let mut output = Vec::new();
    let mut stack = Vec::new();
    let mut current = Some(value);

    loop {
        if let Some(value) = current.take() {
            match value {
                JsonValue::Null => output.extend_from_slice(b"null"),
                JsonValue::Bool(true) => output.extend_from_slice(b"true"),
                JsonValue::Bool(false) => output.extend_from_slice(b"false"),
                JsonValue::Number(number) => output.extend_from_slice(number.as_bytes()),
                JsonValue::String(string) => write_string(string, &mut output),
                JsonValue::Array(values) => {
                    output.push(b'[');
                    if values.is_empty() {
                        output.push(b']');
                    } else {
                        stack.push(Frame::Array(values, 1));
                        current = Some(&values[0]);
                        continue;
                    }
                }
                JsonValue::Object(members) => {
                    output.push(b'{');
                    if members.is_empty() {
                        output.push(b'}');
                    } else {
                        write_string(&members[0].0, &mut output);
                        output.push(b':');
                        stack.push(Frame::Object(members, 1));
                        current = Some(&members[0].1);
                        continue;
                    }
                }
            }
        }

        let Some(frame) = stack.pop() else {
            return output;
        };
        match frame {
            Frame::Array(values, next) => {
                if next == values.len() {
                    output.push(b']');
                } else {
                    output.push(b',');
                    stack.push(Frame::Array(values, next + 1));
                    current = Some(&values[next]);
                }
            }
            Frame::Object(members, next) => {
                if next == members.len() {
                    output.push(b'}');
                } else {
                    output.push(b',');
                    write_string(&members[next].0, &mut output);
                    output.push(b':');
                    stack.push(Frame::Object(members, next + 1));
                    current = Some(&members[next].1);
                }
            }
        }
    }
}

/// Compute the exact byte length produced by [`to_vec`] without allocating the
/// output buffer. `None` means the platform `usize` length would overflow.
pub(crate) fn encoded_len(value: &JsonValue) -> Option<usize> {
    let mut length = 0_usize;
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        match value {
            JsonValue::Null | JsonValue::Bool(true) => length = length.checked_add(4)?,
            JsonValue::Bool(false) => length = length.checked_add(5)?,
            JsonValue::Number(number) => length = length.checked_add(number.len())?,
            JsonValue::String(string) => {
                length = length.checked_add(encoded_string_len(string)?)?;
            }
            JsonValue::Array(values) => {
                length = length.checked_add(2)?;
                length = length.checked_add(values.len().saturating_sub(1))?;
                stack.extend(values.iter().rev());
            }
            JsonValue::Object(members) => {
                length = length.checked_add(2)?;
                length = length.checked_add(members.len().saturating_sub(1))?;
                length = length.checked_add(members.len())?;
                for (key, child) in members.iter().rev() {
                    length = length.checked_add(encoded_string_len(key)?)?;
                    stack.push(child);
                }
            }
        }
    }
    Some(length)
}

fn encoded_string_len(value: &str) -> Option<usize> {
    let mut length = 2_usize;
    for character in value.chars() {
        let bytes = match character {
            '"' | '\\' | '\u{0008}' | '\u{000c}' | '\n' | '\r' | '\t' => 2,
            '\u{0000}'..='\u{001f}' => 6,
            _ => character.len_utf8(),
        };
        length = length.checked_add(bytes)?;
    }
    Some(length)
}

fn write_string(value: &str, output: &mut Vec<u8>) {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    output.push(b'"');
    for character in value.chars() {
        match character {
            '"' => output.extend_from_slice(b"\\\""),
            '\\' => output.extend_from_slice(b"\\\\"),
            '\u{0008}' => output.extend_from_slice(b"\\b"),
            '\u{000c}' => output.extend_from_slice(b"\\f"),
            '\n' => output.extend_from_slice(b"\\n"),
            '\r' => output.extend_from_slice(b"\\r"),
            '\t' => output.extend_from_slice(b"\\t"),
            '\u{0000}'..='\u{001f}' => {
                let byte = character as u8;
                output.extend_from_slice(b"\\u00");
                output.push(HEX[usize::from(byte >> 4)]);
                output.push(HEX[usize::from(byte & 0x0f)]);
            }
            _ => {
                let mut buffer = [0_u8; 4];
                output.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    output.push(b'"');
}

#[cfg(test)]
mod tests {
    use super::{JsonErrorKind, JsonValue, Limits, encoded_len, parse, parse_with_limits, to_vec};

    const TEST_LIMITS: Limits = Limits {
        input_bytes: usize::MAX,
        string_bytes: usize::MAX,
        collection_items: usize::MAX,
        total_values: usize::MAX,
        depth: 512,
    };

    #[test]
    fn parses_structure_and_writes_compact_json() {
        let source = br#" { "a" : [null, true, false], "n": -0.1200e+03 } "#;
        let value = parse(source).unwrap();
        assert_eq!(
            value,
            JsonValue::Object(vec![
                (
                    "a".to_owned(),
                    JsonValue::Array(vec![
                        JsonValue::Null,
                        JsonValue::Bool(true),
                        JsonValue::Bool(false),
                    ]),
                ),
                ("n".to_owned(), JsonValue::Number("-0.1200e+03".to_owned()),),
            ])
        );
        assert_eq!(
            to_vec(&value),
            br#"{"a":[null,true,false],"n":-0.1200e+03}"#
        );
    }

    #[test]
    fn decodes_all_string_escapes_and_canonicalizes_them() {
        let value = parse(br#""\"\\\/\b\f\n\r\t\u0000""#).unwrap();
        assert_eq!(
            value,
            JsonValue::String("\"\\/\u{0008}\u{000c}\n\r\t\0".to_owned())
        );
        assert_eq!(to_vec(&value), br#""\"\\/\b\f\n\r\t\u0000""#);
    }

    #[test]
    fn accepts_raw_unicode_bmp_escapes_and_surrogate_pairs() {
        let value = parse(r#""한글 \u00e9 \uD83D\uDE80""#.as_bytes()).unwrap();
        assert_eq!(value, JsonValue::String("한글 é 🚀".to_owned()));
        assert_eq!(to_vec(&value), "\"한글 é 🚀\"".as_bytes());
    }

    #[test]
    fn preserves_duplicate_object_keys_and_order() {
        let value = parse(br#"{"x":1,"x":2,"a":3}"#).unwrap();
        assert_eq!(
            value,
            JsonValue::Object(vec![
                ("x".to_owned(), JsonValue::Number("1".to_owned())),
                ("x".to_owned(), JsonValue::Number("2".to_owned())),
                ("a".to_owned(), JsonValue::Number("3".to_owned())),
            ])
        );
        assert_eq!(to_vec(&value), br#"{"x":1,"x":2,"a":3}"#);
    }

    #[test]
    fn preserves_every_valid_number_form() {
        for number in [
            "0", "-0", "1", "-17", "0.0", "12.3400", "1e0", "1E+09", "-3.2e-7",
        ] {
            let value = parse(number.as_bytes()).unwrap();
            assert_eq!(value, JsonValue::Number(number.to_owned()));
            assert_eq!(to_vec(&value), number.as_bytes());
        }
    }

    #[test]
    fn rejects_invalid_numbers() {
        for number in [
            "00", "-01", "1.", "1e", "1e+", "--1", ".1", "+1", "NaN", "Infinity",
        ] {
            assert!(parse(number.as_bytes()).is_err(), "accepted {number}");
        }
    }

    #[test]
    fn rejects_unpaired_and_malformed_surrogates() {
        for source in [
            br#""\uD800""#.as_slice(),
            br#""\uDC00""#.as_slice(),
            br#""\uD800\u0041""#.as_slice(),
            br#""\uD800x""#.as_slice(),
            br#""\uD800\uDC0x""#.as_slice(),
        ] {
            assert!(parse(source).is_err());
        }
    }

    #[test]
    fn rejects_invalid_utf8_escapes_controls_and_trailing_data() {
        let cases: &[&[u8]] = &[
            &[b'"', 0xff, b'"'],
            br#""\x20""#,
            b"\"line\nfeed\"",
            br#""\u12xz""#,
            b"true false",
            b"[1,]",
            b"{\"a\":1,}",
            b"{a:1}",
            b"",
        ];
        for source in cases {
            assert!(parse(source).is_err(), "accepted {source:?}");
        }
    }

    #[test]
    fn permits_512_collection_levels_and_rejects_513() {
        let mut accepted = "[".repeat(512);
        accepted.push('0');
        accepted.push_str(&"]".repeat(512));
        assert!(parse(accepted.as_bytes()).is_ok());

        let mut rejected = "[".repeat(513);
        rejected.push('0');
        rejected.push_str(&"]".repeat(513));
        let error = parse(rejected.as_bytes()).unwrap_err();
        assert_eq!(error.kind, JsonErrorKind::NestingTooDeep);
    }

    #[test]
    fn reports_each_resource_limit_explicitly() {
        let mut limits = TEST_LIMITS;
        limits.input_bytes = 3;
        assert_eq!(
            parse_with_limits(b"null", limits).unwrap_err().kind,
            JsonErrorKind::InputTooLarge
        );

        let mut limits = TEST_LIMITS;
        limits.string_bytes = 3;
        assert_eq!(
            parse_with_limits(br#""four""#, limits).unwrap_err().kind,
            JsonErrorKind::StringTooLong
        );

        let mut limits = TEST_LIMITS;
        limits.collection_items = 1;
        assert_eq!(
            parse_with_limits(b"[0,1]", limits).unwrap_err().kind,
            JsonErrorKind::CollectionTooLarge
        );

        let mut limits = TEST_LIMITS;
        limits.total_values = 2;
        assert_eq!(
            parse_with_limits(b"[0,1]", limits).unwrap_err().kind,
            JsonErrorKind::TooManyValues
        );

        let mut limits = TEST_LIMITS;
        limits.depth = 1;
        assert_eq!(
            parse_with_limits(b"[[0]]", limits).unwrap_err().kind,
            JsonErrorKind::NestingTooDeep
        );
    }

    #[test]
    fn error_exposes_the_source_byte_offset() {
        let error = parse(b"[0 1]").unwrap_err();
        assert_eq!(error.offset(), 3);
        assert!(error.to_string().contains("byte 3"));
    }

    #[test]
    fn serializer_handles_manually_built_deep_values_without_recursion() {
        let mut value = JsonValue::Null;
        for _ in 0..2_000 {
            value = JsonValue::Array(vec![value]);
        }
        let bytes = to_vec(&value);
        assert_eq!(bytes.len(), 4_004);
        assert_eq!(&bytes[..3], b"[[[");
        assert_eq!(&bytes[bytes.len() - 3..], b"]]]");
    }

    #[test]
    fn encoded_length_matches_the_serializer_including_escapes() {
        let value = parse(r#"{"control":"\u0000\n","unicode":"한글 🚀","a":[true,-0]}"#.as_bytes())
            .unwrap();
        assert_eq!(encoded_len(&value), Some(to_vec(&value).len()));
    }
}
