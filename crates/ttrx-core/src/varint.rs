use crate::codec::CodecError;

pub fn write_u64(mut value: u64, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value.to_le_bytes()[0] & 0x7f) | 0x80);
        value >>= 7;
    }
    output.push(value.to_le_bytes()[0]);
}

pub fn write_i64(value: i64, output: &mut Vec<u8>) {
    let zigzag = (value.unsigned_abs() << 1).wrapping_sub(u64::from(value.is_negative()));
    write_u64(zigzag, output);
}

/// Read one canonical unsigned LEB128 value.
///
/// # Errors
///
/// Returns an error when the value is truncated, overflows 64 bits, or uses a
/// non-canonical encoding.
pub fn read_u64(input: &[u8], cursor: &mut usize) -> Result<u64, CodecError> {
    let mut value = 0_u64;
    for shift in (0..=63).step_by(7) {
        let byte = *input.get(*cursor).ok_or(CodecError::UnexpectedEof)?;
        *cursor += 1;
        let payload = u64::from(byte & 0x7f);
        if shift == 63 && payload > 1 {
            return Err(CodecError::InvalidVarint);
        }
        value |= payload << shift;
        if byte & 0x80 == 0 {
            if shift != 0 && payload == 0 {
                return Err(CodecError::NonCanonicalVarint);
            }
            return Ok(value);
        }
    }
    Err(CodecError::InvalidVarint)
}

/// Read one signed integer encoded with zigzag followed by unsigned LEB128.
///
/// # Errors
///
/// Returns an error when the underlying unsigned varint is invalid.
pub fn read_i64(input: &[u8], cursor: &mut usize) -> Result<i64, CodecError> {
    let value = read_u64(input, cursor)?;
    let magnitude = (value >> 1) + (value & 1);
    if value & 1 == 0 {
        i64::try_from(magnitude).map_err(|_| CodecError::InvalidVarint)
    } else if magnitude == 1_u64 << 63 {
        Ok(i64::MIN)
    } else {
        i64::try_from(magnitude)
            .map(|magnitude| -magnitude)
            .map_err(|_| CodecError::InvalidVarint)
    }
}

#[cfg(test)]
mod tests {
    use super::{read_i64, read_u64, write_i64, write_u64};

    #[test]
    fn unsigned_boundaries_round_trip() {
        for value in [0, 1, 127, 128, 16_384, u64::from(u32::MAX), u64::MAX] {
            let mut bytes = Vec::new();
            write_u64(value, &mut bytes);
            let mut cursor = 0;
            assert_eq!(read_u64(&bytes, &mut cursor).unwrap(), value);
            assert_eq!(cursor, bytes.len());
        }
    }

    #[test]
    fn signed_boundaries_round_trip() {
        for value in [i64::MIN, -1, 0, 1, i64::MAX] {
            let mut bytes = Vec::new();
            write_i64(value, &mut bytes);
            let mut cursor = 0;
            assert_eq!(read_i64(&bytes, &mut cursor).unwrap(), value);
        }
    }

    #[test]
    fn rejects_noncanonical_unsigned_encodings() {
        for bytes in [&[0x80, 0x00][..], &[0x81, 0x00][..], &[0xff, 0x00][..]] {
            let mut cursor = 0;
            assert_eq!(
                read_u64(bytes, &mut cursor),
                Err(crate::codec::CodecError::NonCanonicalVarint)
            );
        }
    }

    #[test]
    fn rejects_truncated_and_overflowing_unsigned_encodings() {
        let mut cursor = 0;
        assert_eq!(
            read_u64(&[0x80], &mut cursor),
            Err(crate::codec::CodecError::UnexpectedEof)
        );

        let mut cursor = 0;
        assert_eq!(
            read_u64(
                &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02],
                &mut cursor
            ),
            Err(crate::codec::CodecError::InvalidVarint)
        );
    }

    #[test]
    fn signed_reader_rejects_noncanonical_underlying_varint() {
        let mut cursor = 0;
        assert_eq!(
            read_i64(&[0x81, 0x00], &mut cursor),
            Err(crate::codec::CodecError::NonCanonicalVarint)
        );
    }
}
