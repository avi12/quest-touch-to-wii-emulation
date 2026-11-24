//! CRC-32 (IEEE 802.3, as zlib computes it) over a DSU datagram.

/// The reflected IEEE polynomial.
const POLYNOMIAL: u32 = 0xedb8_8320;
/// One table entry per byte value.
const TABLE_LEN: usize = 256;
/// Bits shifted through per byte.
const BITS_PER_BYTE: u32 = 8;
/// The register's starting value, also XOR-ed into the result.
const ALL_ONES: u32 = u32::MAX;

/// The CRC-32 of a datagram, as written into its header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Crc32(pub(crate) u32);

/// The remainder of one byte value: eight shifts through the polynomial.
const fn byte_remainder(byte: u32) -> u32 {
    let mut remainder = byte;
    let mut shifts = 0;
    loop {
        let is_done = shifts == BITS_PER_BYTE;
        if is_done {
            return remainder;
        }
        let is_low_bit_set = remainder & 1 == 1;
        remainder = if is_low_bit_set {
            (remainder >> 1) ^ POLYNOMIAL
        } else {
            remainder >> 1
        };
        shifts += 1;
    }
}

/// The remainder of every byte value, computed at compile time.
const TABLE: [u32; TABLE_LEN] = build_table();

const fn build_table() -> [u32; TABLE_LEN] {
    let mut table = [0; TABLE_LEN];
    let mut rest = table.as_mut_slice();
    let mut byte = 0;
    while let Some((entry, tail)) = rest.split_first_mut() {
        *entry = byte_remainder(byte);
        byte += 1;
        rest = tail;
    }
    table
}

impl Crc32 {
    /// The checksum of `bytes`.
    pub(crate) fn compute(bytes: &[u8]) -> Self {
        let register = bytes.iter().fold(ALL_ONES, |register, byte| {
            let [low, ..] = (register ^ u32::from(*byte)).to_le_bytes();
            // A byte always indexes the 256-entry table.
            let remainder = TABLE.get(usize::from(low)).copied().unwrap_or_default();
            (register >> BITS_PER_BYTE) ^ remainder
        });
        Self(register ^ ALL_ONES)
    }
}

#[cfg(test)]
mod tests {
    use crate::wire::crc::Crc32;

    /// The standard CRC-32 check value: the checksum of "123456789".
    const CHECK_VALUE: u32 = 0xcbf4_3926;

    #[test]
    fn matches_the_standard_check_value() {
        assert_eq!(Crc32::compute(b"123456789"), Crc32(CHECK_VALUE));
    }

    #[test]
    fn empty_input_is_zero() {
        assert_eq!(Crc32::compute(&[]), Crc32(0));
    }
}
