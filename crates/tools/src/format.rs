//! Number formatting that matches JavaScript's, so the tools print what the TypeScript ones did.

/// Enough fraction digits to print any `f64` exactly.
const EXACT_DIGITS: usize = 1100;
/// JavaScript switches to exponent notation at or above this magnitude.
const EXPONENT_ABOVE: f64 = 1e21;
/// ... and below this one.
const EXPONENT_BELOW: f64 = 1e-6;

/// `Number.prototype.toFixed`: rounds the exact value half away from zero.
pub fn to_fixed(value: f64, digits: usize) -> String {
    if !value.is_finite() || value.abs() >= EXPONENT_ABOVE {
        return js_number(value);
    }
    let exact = format!("{:.EXACT_DIGITS$}", value.abs());
    let (whole, fraction) = exact.split_once('.').unwrap_or((&exact, ""));
    let kept = fraction.get(..digits).unwrap_or(fraction);
    let rounds_up = fraction
        .as_bytes()
        .get(digits)
        .is_some_and(|digit| *digit >= b'5');
    let mut decimal: Vec<u8> = whole.bytes().chain(kept.bytes()).collect();
    if rounds_up {
        increment(&mut decimal);
    }
    let split = decimal.len().saturating_sub(digits);
    let (integer, fractional) = decimal.split_at(split);
    let sign = if value < 0.0 { "-" } else { "" };
    let integer = String::from_utf8_lossy(integer);
    if digits == 0 {
        return format!("{sign}{integer}");
    }
    format!("{sign}{integer}.{}", String::from_utf8_lossy(fractional))
}

/// Adds one to the last digit of an ASCII decimal, carrying as needed.
fn increment(decimal: &mut Vec<u8>) {
    for digit in decimal.iter_mut().rev() {
        if *digit == b'9' {
            *digit = b'0';
            continue;
        }
        *digit += 1;
        return;
    }
    decimal.insert(0, b'1');
}

/// `String(number)`: the shortest round-trip form, `0` for negative zero.
pub fn js_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        let sign = if value < 0.0 { "-" } else { "" };
        return format!("{sign}Infinity");
    }
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs();
    if !(EXPONENT_BELOW..EXPONENT_ABOVE).contains(&magnitude) {
        let exponent = format!("{value:e}");
        return match exponent.split_once('e') {
            Some((mantissa, power)) if !power.starts_with('-') => format!("{mantissa}e+{power}"),
            _ => exponent,
        };
    }
    value.to_string()
}

/// `String(Number(value.toFixed(digits)) + 0)`: rounded, then printed in the shortest form.
pub fn rounded(value: f64, digits: usize) -> String {
    let fixed = to_fixed(value, digits);
    fixed
        .parse::<f64>()
        .map_or(fixed, |number| js_number(number + 0.0))
}

/// `[a b c]` with each component [`rounded`].
pub fn rounded_vector(values: &[f64], digits: usize) -> String {
    let parts: Vec<String> = values.iter().map(|value| rounded(*value, digits)).collect();
    format!("[{}]", parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_ties_away_from_zero_like_javascript() {
        assert_eq!(to_fixed(0.125, 2), "0.13");
        assert_eq!(to_fixed(-0.125, 2), "-0.13");
        assert_eq!(to_fixed(0.25, 1), "0.3");
        assert_eq!(to_fixed(1.005, 2), "1.00"); // 1.005 is just below in binary
        assert_eq!(to_fixed(9.999, 2), "10.00");
        assert_eq!(to_fixed(-0.001, 2), "-0.00");
        assert_eq!(to_fixed(-0.0, 2), "0.00");
        assert_eq!(to_fixed(2.5, 0), "3");
        assert_eq!(to_fixed(12.0, 1), "12.0");
    }

    #[test]
    fn prints_numbers_like_javascript() {
        assert_eq!(js_number(5.0), "5");
        assert_eq!(js_number(-0.0), "0");
        assert_eq!(js_number(-20.78), "-20.78");
        assert_eq!(js_number(1e-7), "1e-7");
        assert_eq!(js_number(1e21), "1e+21");
        assert_eq!(rounded(-0.0004, 3), "0");
        assert_eq!(rounded(0.6101, 3), "0.61");
        assert_eq!(rounded_vector(&[0.0, -0.6101, -0.79], 3), "[0 -0.61 -0.79]");
    }
}
