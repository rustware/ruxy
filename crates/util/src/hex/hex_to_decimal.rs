#[inline]
pub fn hex_to_decimal(digit: u8) -> Option<u8> {
  match digit {
    b'0'..=b'9' => Some(digit - b'0'),
    b'A'..=b'F' => Some(digit - b'A' + 10),
    b'a'..=b'f' => Some(digit - b'a' + 10),
    _ => None,
  }
}
