//! Small dependency-free CRC-32C (Castagnoli) implementation.

const POLYNOMIAL: u32 = 0x82f6_3b78;
const TABLE: [u32; 256] = make_table();

const fn make_table() -> [u32; 256] {
    let mut table = [0_u32; 256];
    let mut index = 0_u16;
    while index < 256 {
        let mut crc = index as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = (crc >> 1) ^ (POLYNOMIAL & (0_u32.wrapping_sub(crc & 1)));
            bit += 1;
        }
        table[index as usize] = crc;
        index += 1;
    }
    table
}

#[must_use]
pub fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for &byte in bytes {
        let table_index = usize::from((crc ^ u32::from(byte)).to_le_bytes()[0]);
        crc = (crc >> 8) ^ TABLE[table_index];
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::crc32c;

    #[test]
    fn known_check_value() {
        assert_eq!(crc32c(b"123456789"), 0xe306_9283);
    }
}
