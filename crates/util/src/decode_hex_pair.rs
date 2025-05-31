/// Decodes two bytes holding 4-bits (a hex digit) each into a single byte.
/// This function will return None if either of the bytes holds more than
/// the lower 4 bits.
pub fn decode_hex_pair(high: u8, low: u8) -> Option<u8> {
  fn hex_val(d: u8) -> Option<u8> {
    match d {
      b'0'..=b'9' => Some(d - b'0'),
      b'a'..=b'f' => Some(d - b'a' + 10),
      b'A'..=b'F' => Some(d - b'A' + 10),
      _ => None,
    }
  }

  Some((hex_val(high)? << 4) | hex_val(low)?)
}
