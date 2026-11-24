//! Cosmos `LegacyDec` values, as prices and quantities are written in exchange messages.
//!
//! In JSON and on screen, a `LegacyDec` is a decimal such as `1.5`. On the protobuf wire it is
//! different: the value times 10^18, as a base-10 integer string (`1500000000000000000`).
//! Sending `1.5` in a protobuf message is a common mistake.

use crate::Error;

/// The number of fraction digits of a `LegacyDec`.
pub const PRECISION: usize = 18;

/// The protobuf wire form of the decimal `s`: its value times 10^18, as an integer string.
///
/// `s` must be a non-negative decimal: digits, optionally a dot and at most 18 more digits.
/// No sign, exponent, spaces or digit separators.
pub fn to_wire(s: &str) -> Result<String, Error> {
    let invalid = || Error::InvalidDecimal(s.to_string());
    let (integer, fraction) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };
    let digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
    if integer.is_empty()
        || !digits(integer)
        || !digits(fraction)
        || (s.contains('.') && fraction.is_empty())
        || fraction.len() > PRECISION
    {
        return Err(invalid());
    }
    let mut wire = String::with_capacity(integer.len() + PRECISION);
    wire.push_str(integer);
    wire.push_str(fraction);
    wire.extend(std::iter::repeat('0').take(PRECISION - fraction.len()));
    let trimmed = wire.trim_start_matches('0');
    Ok(if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    })
}
