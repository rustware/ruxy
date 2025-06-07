use super::hex_to_decimal;

/// Decodes two bytes holding 4-bits (a hex digit) each into a single byte.
/// This function will return None if either of the bytes holds more than
/// the lower 4 bits.
pub fn hex_pair_to_decimal(high: u8, low: u8) -> Option<u8> {
  Some((hex_to_decimal(high)? << 4) | hex_to_decimal(low)?)
}
