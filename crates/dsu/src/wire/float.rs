//! `f64` -> `f32` with IEEE round-to-nearest-even, as a JS `writeFloatLE` writes it; the bits are
//! rounded by hand because the workspace converts nothing with `as`.

/// Explicit mantissa bits of an `f64`.
const F64_MANTISSA_BITS: u32 = 52;
/// Exponent bias of an `f64`.
const F64_BIAS: i32 = 1023;
/// Explicit mantissa bits of an `f32`.
const F32_MANTISSA_BITS: u32 = 23;
/// Mantissa bits an `f64` loses on the way to a normal `f32`.
const DROPPED_BITS: u32 = F64_MANTISSA_BITS - F32_MANTISSA_BITS;
/// Smallest unbiased exponent of a normal `f32`.
const F32_MIN_EXPONENT: i32 = -126;
/// Largest unbiased exponent of a finite `f32`.
const F32_MAX_EXPONENT: i32 = 127;
/// The sign bit's position in an `f32`.
const F32_SIGN_SHIFT: u32 = 31;
/// Bits of positive infinity.
const F32_INFINITY_BITS: u32 = 0x7f80_0000;
/// The low bits of the `f64` exponent's high byte, and the shift of its low nibble.
const EXPONENT_HIGH_MASK: u8 = 0x7f;
const NIBBLE_BITS: u32 = 4;
/// The implicit leading bit of a normal `f64` significand.
const IMPLICIT_BIT: u64 = 1 << F64_MANTISSA_BITS;

/// `value` rounded to the nearest `f32`, ties to even; overflow becomes infinity.
pub(crate) fn narrow(value: f64) -> f32 {
    if value.is_nan() {
        return f32::NAN;
    }
    let sign = u32::from(value.is_sign_negative()) << F32_SIGN_SHIFT;
    let [high, next, ..] = value.to_bits().to_be_bytes();
    let biased =
        (u16::from(high & EXPONENT_HIGH_MASK) << NIBBLE_BITS) | u16::from(next >> NIBBLE_BITS);
    let is_zero_or_subnormal = biased == 0;
    if is_zero_or_subnormal {
        // Zero or an f64 subnormal: far below the smallest f32.
        return f32::from_bits(sign);
    }
    let exponent = i32::from(biased) - F64_BIAS;
    let is_overflow = value.is_infinite() || exponent > F32_MAX_EXPONENT;
    if is_overflow {
        return f32::from_bits(sign | F32_INFINITY_BITS);
    }
    let significand = (value.to_bits() & (IMPLICIT_BIT - 1)) | IMPLICIT_BIT;
    let is_normal = exponent >= F32_MIN_EXPONENT;
    let magnitude = if is_normal {
        // The implicit bit lands on the exponent field's lowest bit, hence the field minus one;
        // a carry out of the mantissa rounds up into the exponent (up to exactly infinity).
        let field = (exponent - F32_MIN_EXPONENT).unsigned_abs();
        (field << F32_MANTISSA_BITS) + low_u32(round_shift(significand, DROPPED_BITS))
    } else {
        let shift = DROPPED_BITS + (F32_MIN_EXPONENT - exponent).unsigned_abs();
        low_u32(round_shift(significand, shift))
    };
    f32::from_bits(sign | magnitude)
}

/// `value >> shift`, rounded to nearest with ties to even.
fn round_shift(value: u64, shift: u32) -> u64 {
    let kept = value.checked_shr(shift).unwrap_or(0);
    let dropped = value - kept.checked_shl(shift).unwrap_or(0);
    let half = 1_u64.checked_shl(shift - 1).unwrap_or(u64::MAX);
    let is_odd = kept & 1 == 1;
    let should_round_up = dropped > half || (dropped == half && is_odd);
    kept + u64::from(should_round_up)
}

/// The low 32 bits of a value known to fit them.
fn low_u32(value: u64) -> u32 {
    let [byte0, byte1, byte2, byte3, ..] = value.to_le_bytes();
    u32::from_le_bytes([byte0, byte1, byte2, byte3])
}

#[cfg(test)]
mod tests {
    use crate::wire::float::narrow;

    #[test]
    fn rounds_normals_to_nearest() {
        assert_eq!(narrow(0.1).to_bits(), 0x3dcc_cccd);
        assert_eq!(narrow(-3.3).to_bits(), 0xc053_3333);
        assert_eq!(narrow(f64::from(f32::MAX)), f32::MAX);
    }

    #[test]
    fn ties_go_to_even() {
        assert_eq!(narrow(1.0 + 2_f64.powi(-24)), 1.0);
        assert_eq!(narrow(1.0 + 3.0 * 2_f64.powi(-24)), 1.0 + 2_f32.powi(-22));
    }

    #[test]
    fn handles_subnormals_zero_and_specials() {
        assert_eq!(narrow(f64::from(f32::from_bits(1))).to_bits(), 1);
        assert_eq!(narrow(2_f64.powi(-150)).to_bits(), 0);
        assert_eq!(narrow(1.5 * 2_f64.powi(-150)).to_bits(), 1);
        assert_eq!(narrow(-0.0).to_bits(), 0x8000_0000);
        assert_eq!(narrow(1e-300).to_bits(), 0);
        assert!(narrow(f64::NAN).is_nan());
    }

    #[test]
    fn overflow_becomes_infinity() {
        assert_eq!(narrow(1e40), f32::INFINITY);
        assert_eq!(narrow(-1e40), f32::NEG_INFINITY);
        assert_eq!(narrow(f64::INFINITY), f32::INFINITY);
        assert_eq!(narrow(2_f64.powi(128) - 2_f64.powi(103)), f32::INFINITY);
    }
}
