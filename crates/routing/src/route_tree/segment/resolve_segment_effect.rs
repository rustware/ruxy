use crate::{DynamicSequenceArity, SegmentEffect, UrlMatcherSequence};

const SLOT_START: &str = "@";
const ESCAPE_SEQUENCE_START: char = '%';
const DYNAMIC_SEQUENCE_START: char = '{';
const DYNAMIC_SEQUENCE_END: char = '}';
const ARITY_SPECIFIER_START: char = '[';
const ARITY_SPECIFIER_END: char = ']';
const ROUTE_GROUP_START: char = '(';
const ROUTE_GROUP_END: char = ')';

#[derive(PartialEq)]
enum SpecialChar {
  EscapeSequenceStart,
  DynamicSequenceStart,
  DynamicSequenceEnd,
  AritySpecifierStart,
  AritySpecifierEnd,
  RouteGroupStart,
  RouteGroupEnd,
}

impl SpecialChar {
  fn get(ch: char) -> Option<Self> {
    match ch {
      ESCAPE_SEQUENCE_START => Some(Self::EscapeSequenceStart),
      DYNAMIC_SEQUENCE_START => Some(Self::DynamicSequenceStart),
      DYNAMIC_SEQUENCE_END => Some(Self::DynamicSequenceEnd),
      ARITY_SPECIFIER_START => Some(Self::AritySpecifierStart),
      ARITY_SPECIFIER_END => Some(Self::AritySpecifierEnd),
      ROUTE_GROUP_START => Some(Self::RouteGroupStart),
      ROUTE_GROUP_END => Some(Self::RouteGroupEnd),
      _ => None,
    }
  }
}

/// Route Segment directory naming conventions:
///
/// Dynamic sequence syntax: `{<ident><arity specifier>}`.
///
/// `{var}`:        Simple dynamic sequence replacing `var` with the value from the URL.
///                 This is a shorthand for `{var[1]}`.
///
/// `{vars[n]}`     A sequence capturing exactly `n` URL segments.
///
/// `{vars[n..]}`   A sequence capturing `n` or more URL segments.
///
/// `{vars[n..m]}`  A sequence capturing `n` to `m` URL segments (both inclusive).
///
/// `(name)`        Route Group
///                 This is a shorthand for `{name[0]}`
///
/// `@name`         Named slot
///
/// Rules:
/// 1. Only one dynamic sequence per segment.
/// 2. Dynamic sequence other than arity:1 cannot have prefix or suffix.
/// 3. Dynamic sequence name can contain only a-z, A-Z, 0-9, and _.
/// 4. Special characters (%@(){}[]) must be percent-encoded if wanted to be matched literally.
///
/// Forbidden characters: https://stackoverflow.com/a/31976060
pub fn resolve_segment_effect(dir_name: &str) -> Result<SegmentEffect, String> {
  // Named Slots for Parallel Routes (`@my_slot`)
  if let Some(slot_name) = dir_name.strip_prefix(SLOT_START) {
    let decoded = decode_percent_encodings(slot_name, dir_name)?;
    return Ok(SegmentEffect::Slot { name: decoded });
  }

  // Route Groups – shorthand form (`(group)`)
  if dir_name.starts_with(ROUTE_GROUP_START) && dir_name.ends_with(ROUTE_GROUP_END) {
    return Ok(SegmentEffect::AlwaysMatch);
  }

  let sequences = resolve_url_matcher_sequences(dir_name)?;

  for sequence in sequences.iter() {
    let UrlMatcherSequence::Dynamic { arity, var_name } = sequence else {
      continue;
    };

    if sequences.len() > 1 && !matches!(arity, DynamicSequenceArity::Exact(1)) {
      return Err(format!(
        "Dynamic sequence with segment arity other than 1 cannot have prefix or suffix.\r\n\
        Use `{{{ident}[1]}}` or `{{{ident}}}` shorthand to be able to use a prefix or suffix in the same segment.",
        ident = var_name
      ));
    }

    if matches!(arity, DynamicSequenceArity::Exact(0)) {
      // Sequence with arity 0 makes the segment AlwaysMatch
      return Ok(SegmentEffect::AlwaysMatch);
    }
  }

  Ok(SegmentEffect::UrlMatcher { sequences })
}

fn resolve_url_matcher_sequences(dir_name: &str) -> Result<Vec<UrlMatcherSequence>, String> {
  enum ParsingPhase {
    Prefix,
    DynamicVarName,
    DynamicArityLowerBound,
    DynamicArityUpperBound(String),
    Suffix,
  }

  let mut parsing_phase = ParsingPhase::Prefix;

  let mut prefix = String::new();
  let mut dyn_var_name = String::new();
  let mut dyn_arity_lo = String::new();
  let mut dyn_arity_hi = None;
  let mut suffix = String::new();

  let mut chars = dir_name.chars();

  while let Some(ch) = chars.next() {
    match parsing_phase {
      ParsingPhase::Prefix => {
        let Some(special_char) = SpecialChar::get(ch) else {
          prefix.push(ch);
          continue;
        };

        if special_char == SpecialChar::DynamicSequenceStart {
          parsing_phase = ParsingPhase::DynamicVarName;
          continue;
        }

        if special_char == SpecialChar::EscapeSequenceStart {
          prefix.push(decode_percent_encoding(chars.next(), chars.next(), dir_name)?);
          continue;
        }

        return Err(unexpected_special_char_err(ch));
      }
      ParsingPhase::DynamicVarName => {
        if matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
          dyn_var_name.push(ch);
          continue;
        }

        if dyn_var_name.is_empty() {
          return Err(format!(
            "Unexpected character \"{ch}\" after dynamic sequence opening bracket – `{{`.\r\n\
            Dynamic sequence opening bracket must be followed by a name for this dynamic sequence.\r\n\
            Allowed characters in dynamic sequence names are a-z, A-Z, 0-9 and _."
          ));
        }

        if ch == ARITY_SPECIFIER_START {
          parsing_phase = ParsingPhase::DynamicArityLowerBound;
          continue;
        }

        if ch == DYNAMIC_SEQUENCE_END {
          dyn_arity_lo = String::from("1");
          parsing_phase = ParsingPhase::Suffix;
          continue;
        }

        return Err(format!(
          "Unexpected character \"{ch}\" in dynamic sequence name.\r\n\
          Dynamic sequence name can only contain a-z, A-Z, 0-9 and _, and can only be followed by \
          either a segment arity specifier opening bracket – `[`, or dynamic sequence closing bracket – `}}`."
        ));
      }
      ParsingPhase::DynamicArityLowerBound => {
        if ch.is_ascii_digit() {
          dyn_arity_lo.push(ch);
          continue;
        }

        if dyn_arity_lo.is_empty() {
          return Err(format!(
            "Unexpected character \"{ch}\" after arity specifier opening bracket – `[`.\r\n\
            Arity specifier opening bracket must be followed by a number (either exact or as part of range)."
          ));
        }

        if ch == ARITY_SPECIFIER_END {
          if let Some(next_char) = chars.next() {
            if next_char == DYNAMIC_SEQUENCE_END {
              parsing_phase = ParsingPhase::Suffix;
              continue;
            }

            return Err(unexpected_char_after_arity_close_err(ch));
          }

          return Err(dirname_end_after_arity_close_err());
        }

        if ch == '.' {
          if let Some(next_char) = chars.next() {
            if next_char == '.' {
              dyn_arity_hi = Some(String::new());
              parsing_phase = ParsingPhase::DynamicArityUpperBound(String::new());
              continue;
            }

            return Err(format!(
              "Unexpected character \"{ch}\" after a dot in arity specifier.\r\n\
              Single dot in arity specifier can only be followed by another dot, composing a range operator – `..`.",
            ));
          }

          return Err(
            "Directory name ends after a dot inside arity specifier of the dynamic sequence.\r\n\
            Both arity specifier and the enclosing dynamic sequence must be properly ended."
              .to_string(),
          );
        }

        return Err(format!(
          "Unexpected character \"{ch}\" in arity specifier numeric bound.\r\n\
          Arity specifier numeric bound can only contain numbers, and can only be followed by \
          either an arity specifier closing bracket – `]`, or range operator – `..`."
        ));
      }
      ParsingPhase::DynamicArityUpperBound(ref mut bound) => {
        if ch.is_ascii_digit() {
          bound.push(ch);
          continue;
        }

        if ch == ARITY_SPECIFIER_END {
          if let Some(next_char) = chars.next() {
            if next_char == DYNAMIC_SEQUENCE_END {
              dyn_arity_hi = Some(std::mem::take(bound));
              parsing_phase = ParsingPhase::Suffix;
              continue;
            }

            return Err(unexpected_char_after_arity_close_err(ch));
          }

          return Err(dirname_end_after_arity_close_err());
        }

        if ch == '.' && bound.is_empty() {
          return Err(
            "Unexpected additional dot after range operator specifier upper bound – `..`.\r\n\
            Range operator consists of exactly two dots (`..`) followed by either a number or \
            an arity specifier closing bracket – `]`."
              .to_string(),
          );
        }

        return Err(format!(
          "Unexpected character \"{ch}\" in range arity specifier upper bound.\r\n\
          Arity specifier upper bound can only contain numbers, and can only be followed by \
          an arity specifier closing bracket – `]`."
        ));
      }
      ParsingPhase::Suffix => {
        let Some(special_char) = SpecialChar::get(ch) else {
          suffix.push(ch);
          continue;
        };

        if special_char == SpecialChar::EscapeSequenceStart {
          suffix.push(decode_percent_encoding(chars.next(), chars.next(), dir_name)?);
          continue;
        }

        if special_char == SpecialChar::DynamicSequenceStart {
          return Err(format!(
            "Unexpected character \"{ch}\" in the route segment suffix.\r\n\
            A single route segment can contain only a single dynamic sequence."
          ));
        }

        return Err(unexpected_special_char_err(ch));
      }
    }
  }

  if !matches!(parsing_phase, ParsingPhase::Prefix | ParsingPhase::Suffix) {
    return Err(
      "A dynamic sequence is started with an opening bracket – `{`, but never closed.\r\n\
      Dynamic sequences must be properly closed."
        .to_string(),
    );
  }

  // Prefix, dynamic, suffix = max 3 sequences
  let mut sequences = Vec::with_capacity(3);

  if !prefix.is_empty() {
    sequences.push(UrlMatcherSequence::Literal(prefix));
  }

  // The second condition is just a sanity check, shouldn't really happen
  if !dyn_var_name.is_empty() && !dyn_arity_lo.is_empty() {
    sequences.push(UrlMatcherSequence::Dynamic {
      var_name: dyn_var_name,
      arity: parse_arity(dyn_arity_lo, dyn_arity_hi, dir_name)?,
    });
  }

  if !suffix.is_empty() {
    sequences.push(UrlMatcherSequence::Literal(suffix));
  }

  Ok(sequences)
}

fn parse_arity(lo: String, hi: Option<String>, dir_name: &str) -> Result<DynamicSequenceArity, String> {
  let lo = lo.parse::<usize>().map_err(|_| arity_unsupported_number_err(lo))?;

  let Some(hi) = hi else {
    return Ok(DynamicSequenceArity::Exact(lo));
  };

  if hi.is_empty() {
    return Ok(DynamicSequenceArity::Range(lo, None)); 
  }
  
  let hi = hi.parse::<usize>().map_err(|_| arity_unsupported_number_err(hi))?;

  if lo > hi {
    return Err("Dynamic sequence arity lower bound cannot be greater than the upper bound.".to_string());
  }

  if lo == hi {
    return Ok(DynamicSequenceArity::Exact(lo));
  }

  Ok(DynamicSequenceArity::Range(lo, Some(hi)))
}

fn decode_percent_encoding(ch1: Option<char>, ch2: Option<char>, dir_name: &str) -> Result<char, String> {
  let (Some(ch1), Some(ch2)) = (ch1, ch2) else {
    return Err(
      "Incomplete percent-encoding.\r\n\
      If you want to use a literal percent character, encode it as \"%25\"."
        .to_string(),
    );
  };

  let hex = format!("{}{}", ch1, ch2);

  let byte = u8::from_str_radix(&hex, 16).map_err(|_| {
    format!(
      "Invalid percent-encoding \"%{hex}\".\r\n\
      If you want to use a literal percent character, encode it as \"%25\"."
    )
  })?;

  Ok(byte as char)
}

fn decode_percent_encodings(input: &str, dir_name: &str) -> Result<String, String> {
  let mut output = String::with_capacity(input.len());
  let mut chars = input.chars();

  while let Some(ch) = chars.next() {
    if ch != ESCAPE_SEQUENCE_START {
      output.push(ch);
      continue;
    }

    let decoded = decode_percent_encoding(chars.next(), chars.next(), dir_name)?;

    output.push(decoded);
  }

  Ok(output)
}

fn unexpected_special_char_err(ch: char) -> String {
  format!("Unexpected special character \"{ch}\". You might want to percent-encode it as \"%{:02X}\".", ch as u8)
}

fn unexpected_char_after_arity_close_err(ch: char) -> String {
  format!(
    "Unexpected character \"{ch}\" in dynamic sequence after the arity specifier closing bracket – `]`.\r\n\
    Arity specifier closing bracket can only be followed by the dynamic sequence closing bracket – `}}`."
  )
}

fn dirname_end_after_arity_close_err() -> String {
  "Directory name ends after the arity specifier closing bracket – `]`.\r\n\
    Did you forget to include a `}` to end the enclosing dynamic sequence?"
    .to_string()
}

fn arity_unsupported_number_err(num: String) -> String {
  format!("Dynamic sequence contains segment arity with unsupported number: \"{num}\".",)
}
