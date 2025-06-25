use std::borrow::Cow;
use std::string::FromUtf8Error;

use crate::hex::hex_to_decimal;

/// Decode dollar-encoded string assuming UTF-8 encoding.
///
/// Avoids allocations if no decoding is needed. For this reason returns a Cow.
/// If you need a `String`, call `.into_owned()` (not `.to_owned()`).
///
/// Derived from the implementation of the `urlencoding` crate:
/// https://crates.io/crates/urlencoding
pub fn decode(data: &str) -> Result<Cow<'_, str>, FromUtf8Error> {
  match decode_binary(data.as_bytes()) {
    Cow::Borrowed(_) => Ok(Cow::Borrowed(data)),
    Cow::Owned(s) => Ok(Cow::Owned(String::from_utf8(s)?)),
  }
}

/// Decode dollar-encoded string as binary data, in any encoding.
pub fn decode_binary(data: &[u8]) -> Cow<'_, [u8]> {
  let offset = data.iter().take_while(|&&c| c != b'$').count();

  if offset >= data.len() {
    return Cow::Borrowed(data);
  }

  let mut decoded: Vec<u8> = Vec::with_capacity(data.len());
  let mut out = NeverRealloc(&mut decoded);

  let (ascii, mut data) = data.split_at(offset);
  out.extend_from_slice(ascii);

  loop {
    let mut parts = data.splitn(2, |&c| c == b'$');

    // first the decoded non-$ part
    let non_escaped_part = parts.next().unwrap();
    let rest = parts.next();

    if rest.is_none() && out.0.is_empty() {
      // if empty there were no '$' in the string
      return data.into();
    }

    out.extend_from_slice(non_escaped_part);

    // then decode one $xx
    match rest {
      Some(rest) => match rest.get(0..2) {
        Some(&[first, second]) => match hex_to_decimal(first) {
          Some(first_val) => match hex_to_decimal(second) {
            Some(second_val) => {
              out.push((first_val << 4) | second_val);
              data = &rest[2..];
            }
            None => {
              out.extend_from_slice(&[b'$', first]);
              data = &rest[1..];
            }
          },
          None => {
            out.push(b'$');
            data = rest;
          }
        },
        _ => {
          // too short
          out.push(b'$');
          out.extend_from_slice(rest);
          break;
        }
      },
      None => break,
    }
  }
  Cow::Owned(decoded)
}

struct NeverRealloc<'a, T>(pub &'a mut Vec<T>);

impl<T> NeverRealloc<'_, T> {
  #[inline]
  pub fn push(&mut self, val: T) {
    // these branches only exist to remove redundant reallocation code
    // (the capacity is always sufficient)
    if self.0.len() != self.0.capacity() {
      self.0.push(val);
    }
  }
  #[inline]
  pub fn extend_from_slice(&mut self, val: &[T])
  where
    T: Clone,
  {
    if self.0.capacity() - self.0.len() >= val.len() {
      self.0.extend_from_slice(val);
    }
  }
}

#[test]
fn decodes() {
  assert!(matches!(decode("hello"), Ok(Cow::Borrowed("hello"))));
  assert!(matches!(decode("hello$20"), Ok(Cow::Owned(s)) if s == "hello "));
  assert!(matches!(decode("$20hello"), Ok(Cow::Owned(s)) if s == " hello"));
}
