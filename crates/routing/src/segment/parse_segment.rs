use ::ruxy_util::dollar_encoding;

use crate::segment::{Arity, DynamicSequence, SegmentEffect, TypedSequence, UrlMatcherSequence};

const SLOT_START: char = '@';
const CUSTOM_MATCH_START: char = '~';
const ESCAPE_SEQUENCE_START: char = '$';
const DYNAMIC_SEQUENCE_START: char = '{';
const DYNAMIC_SEQUENCE_END: char = '}';
const SEGCOUNT_SPECIFIER_START: char = '[';
const SEGCOUNT_SPECIFIER_END: char = ']';
const CHARLEN_SPECIFIER_START: char = '(';
const CHARLEN_SPECIFIER_END: char = ')';
const ROUTE_GROUP_START: char = '(';
const ROUTE_GROUP_END: char = ')';
const EMPTY_SEGMENT: char = '_';

// `[` and `]` are used by Ruxy but not reserved as a special character right now
// because they can only appear inside `{` and `}` which are reserved delimiters.

// `@` is currently only reserved on the beginning of a directory name, otherwise
// ignored and matched literally.

// Possible special characters for future use that are valid in directory names:
// ! # & + , - . = ^ ` ~ [ ] @

// `#` would be great fit for Fragment segments

// `%` is not a Ruxy special characters but we don't want to ever reserve it as
// it would make matching URL-encoded characters more difficult for users.
// Currently they can write `a%2Fb` into directory name and directly match an URL
// that has the same encoded character in it (this decodes to `a/b` but that's not
// Ruxy's concern).

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
      SEGCOUNT_SPECIFIER_START => Some(Self::AritySpecifierStart),
      SEGCOUNT_SPECIFIER_END => Some(Self::AritySpecifierEnd),
      ROUTE_GROUP_START => Some(Self::RouteGroupStart),
      ROUTE_GROUP_END => Some(Self::RouteGroupEnd),
      _ => None,
    }
  }
}

/// Route Segment directory naming conventions:
///
/// Dynamic sequence syntax: `{<ident><segment count specifier><character length specifier>}`,
/// where both the Segment Count Specifier and Character Length Specifier are optional.
///
/// `{foo}`:            Simple dynamic sequence replacing `foo` with the value from the URL.
///                     This is a shorthand for `{foo[1](1..)}`, which means that the dynamic sequence
///                     will match exactly one URL segment containing at least one character.
///
/// `{foo[n]}`          A sequence matching exactly `n` URL segments.
///
/// `{foo[n..]}`        A sequence matching `n` or more URL segments.
///
/// `{foo[n..m]}`       A sequence matching `n` to `m` URL segments (both inclusive).
///
/// `{foo(n)}`          A sequence matching exactly `n` characters in each matched segment.
///
/// `{foo(n..)}`        A sequence matching `n` or more characters in each matched segment.
///
/// `{foo(n..m)}`       A sequence matching `n` to `m` characters in each matched segment (both inclusive).
///
/// `{foo[n..](m..)}`   A sequence matching `n` or more segments with `m` or more characters each.
///
/// `(foo)`             Route Group
///                     This is a shorthand for `{foo[0]}`
///
/// `_`                 Empty Segment
///                     This is a shorthand for `{foo[1](0)}`
///
/// `@foo`              Named Slot for Parallel Routing
///
/// `#foo`              Named Fragment for Composite Routing
///
/// `~foo`              Custom Match segment
///
/// Rules:
/// 1. Only one Dynamic Sequence per Route Segment.
/// 2. Dynamic sequence other than arity:1 cannot have prefix or suffix.
/// 3. Dynamic sequence name can contain only a-z, A-Z, 0-9, and _.
/// 4. Special characters in segment dirnames must be dollar-encoded if wanted to be matched literally.
///
/// Forbidden characters: https://stackoverflow.com/a/31976060
pub fn parse_segment(dir_name: &str) -> Result<SegmentEffect, String> {
  // Named Slots for Parallel Routes (`@my_slot`)
  if let Some(slot_name) = dir_name.strip_prefix(SLOT_START) {
    validate_slot_name(slot_name)?;
    return Ok(SegmentEffect::Slot { name: slot_name.into() });
  }

  // Custom Match segments (~my_dir)
  if let Some(identifier) = dir_name.strip_prefix(CUSTOM_MATCH_START) {
    validate_custom_match_identifier(identifier)?;
    return Ok(SegmentEffect::CustomMatch { identifier: identifier.into() });
  }

  // Route Groups – shorthand form (`(group)`)
  if dir_name.starts_with(ROUTE_GROUP_START) && dir_name.ends_with(ROUTE_GROUP_END) {
    return Ok(SegmentEffect::Group);
  }

  // Empty segments (`_`)
  if dir_name.starts_with(EMPTY_SEGMENT) && dir_name.len() == 1 {
    return Ok(SegmentEffect::EmptySegment);
  }

  let sequences = parse_sequences(dir_name)?;

  for sequence in sequences.iter() {
    let UrlMatcherSequence { typed: TypedSequence::Dynamic(dyn_seq), .. } = sequence else {
      continue;
    };

    if matches!(dyn_seq.seg_count, Arity::Exact(0)) {
      // Sequence with segment count 0 makes the segment effectively a Group Segment
      return Ok(SegmentEffect::Group);
    }

    if matches!(dyn_seq.seg_count, Arity::Exact(1)) && matches!(dyn_seq.char_len, Arity::Exact(0)) {
      // Sequence with segment count 1 and character length 0 makes the segment effectively an Empty Segment
      return Ok(SegmentEffect::EmptySegment);
    }
  }

  Ok(SegmentEffect::UrlMatcher { sequences })
}

fn parse_sequences(dir_name: &str) -> Result<Vec<UrlMatcherSequence>, String> {
  enum ParsingState {
    Literal { seq_start: usize, literal: String },
    DynParamName { seq_start: usize, name: String },
    DynSegCountMin { seq_start: usize, seq: DynamicSequence, count_min: String },
    DynSegCountMax { seq_start: usize, seq: DynamicSequence, count_max: String },
    DynCharLenMin { seq_start: usize, seq: DynamicSequence, len_min: String },
    DynCharLenMax { seq_start: usize, seq: DynamicSequence, len_max: String },
  }

  let mut parsing_state = ParsingState::Literal { seq_start: 0, literal: String::new() };
  let mut parsed_sequences: Vec<UrlMatcherSequence> = Vec::new();

  let mut chars = dir_name.chars().enumerate();

  while let Some((index, ch)) = chars.next() {
    match &mut parsing_state {
      ParsingState::Literal { seq_start, literal } => {
        let Some(special_char) = SpecialChar::get(ch) else {
          literal.push(ch);
          continue;
        };

        if special_char == SpecialChar::EscapeSequenceStart {
          let hex1 = chars.next().map(|(_, hex)| hex);
          let hex2 = chars.next().map(|(_, hex)| hex);
          literal.push_str(&decode_escape_sequence(hex1, hex2, index)?);
          continue;
        }

        if special_char == SpecialChar::DynamicSequenceStart {
          if !literal.is_empty() {
            let typed = TypedSequence::Literal(std::mem::take(literal));
            let sequence = UrlMatcherSequence { start_pos: *seq_start, typed };
            parsed_sequences.push(sequence);
          }

          parsing_state = ParsingState::DynParamName { seq_start: index, name: String::new() };

          continue;
        }

        return Err(unexpected_special_char_err(ch, index));
      }
      ParsingState::DynParamName { seq_start, name } => {
        if matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
          name.push(ch);
          continue;
        }

        if name.is_empty() {
          return Err(format!(
            "Unexpected character \"{ch}\" at position {index} after dynamic sequence opening bracket – `{{`.\r\n\
            Dynamic sequence opening bracket must be followed by an identifier of this dynamic sequence.\r\n\
            Allowed characters in dynamic sequence identifiers are a-z, A-Z, 0-9 and _."
          ));
        }

        let param_name = std::mem::take(name);

        if ch == SEGCOUNT_SPECIFIER_START {
          let seq = DynamicSequence { param_name, ..Default::default() };
          parsing_state = ParsingState::DynSegCountMin { seq_start: *seq_start, seq, count_min: String::new() };
          continue;
        }

        if ch == CHARLEN_SPECIFIER_START {
          let seq = DynamicSequence { param_name, ..Default::default() };
          parsing_state = ParsingState::DynCharLenMin { seq_start: *seq_start, seq, len_min: String::new() };
          continue;
        }

        if ch == DYNAMIC_SEQUENCE_END {
          let sequence = TypedSequence::Dynamic(DynamicSequence { param_name, ..Default::default() });
          parsed_sequences.push(UrlMatcherSequence { start_pos: *seq_start, typed: sequence });
          parsing_state = ParsingState::Literal { seq_start: index + 1, literal: String::new() };
          continue;
        }

        return Err(format!(
          "Unexpected character \"{ch}\" at position {index} (in a dynamic sequence identifier).\r\n\
          Dynamic sequence name can only contain a-z, A-Z, 0-9 and _, and can only be followed by \
          a Segment Count specifier opening bracket – `[`, a Character Length opening bracket – `(`, \
          or the dynamic sequence closing bracket – `}}`."
        ));
      }
      ParsingState::DynSegCountMin { seq_start, seq, count_min } => {
        if ch.is_ascii_digit() {
          count_min.push(ch);
          continue;
        }

        if count_min.is_empty() {
          return Err(format!(
            "Unexpected character \"{ch}\" at position {index} (after Segment Count specifier opening bracket – `[`).\r\n\
            Segment Count specifier opening bracket must be followed by a number (exact or part of a range)."
          ));
        }

        let count_min = std::mem::take(count_min);
        let parsed = count_min.parse::<usize>().map_err(|_| arity_unsupported_digit_err(count_min))?;

        let mut seq = std::mem::take(seq);

        if ch == '.' {
          if let Some((_, next_char)) = chars.next() {
            if next_char == '.' {
              seq.seg_count = Arity::Range(parsed, None);
              parsing_state = ParsingState::DynSegCountMax { seq_start: *seq_start, seq, count_max: String::new() };
              continue;
            }

            return Err(format!(
              "Unexpected character \"{ch}\" after a dot in Segment Count specifier.\r\n\
              Single dot in Segment Count specifier can only be followed by another dot, composing a range operator – `..`.",
            ));
          }

          return Err(String::from(
            "Directory name ends after a dot inside Segment Count specifier of the dynamic sequence.\r\n\
            Both Segment Count specifier and the enclosing dynamic sequence must be properly ended.",
          ));
        }

        if ch == SEGCOUNT_SPECIFIER_END {
          if let Some((_, next_char)) = chars.next() {
            seq.seg_count = Arity::Exact(parsed);

            if next_char == CHARLEN_SPECIFIER_START {
              parsing_state = ParsingState::DynCharLenMin { seq_start: *seq_start, seq, len_min: String::new() };
              continue;
            }

            if next_char == DYNAMIC_SEQUENCE_END {
              parsed_sequences.push(UrlMatcherSequence { start_pos: *seq_start, typed: TypedSequence::Dynamic(seq) });
              parsing_state = ParsingState::Literal { seq_start: index + 1, literal: String::new() };
              continue;
            }

            return Err(unexpected_char_after_segcount_close_err(ch, index + 1));
          }

          return Err(dirname_end_after_segcount_close_err());
        }

        return Err(format!(
          "Unexpected character \"{ch}\" in Segment Count specifier numeric bound.\r\n\
          Segment Count specifier numeric bound can only contain numbers, and can only be followed by \
          either a Segment Count specifier closing bracket – `]`, or range operator – `..`."
        ));
      }
      ParsingState::DynSegCountMax { seq_start, seq, count_max } => {
        if ch.is_ascii_digit() {
          count_max.push(ch);
          continue;
        }

        if ch == '.' && count_max.is_empty() {
          return Err(String::from(
            "Unexpected additional dot after Segment Count range operator – `..`.\r\n\
            Segment Count range operator consists of exactly two dots (`..`) followed by either a number or \
            a Segment Count specifier closing bracket – `]`.",
          ));
        }

        let count_max = std::mem::take(count_max);
        let mut seq = std::mem::take(seq);

        let max: Option<usize> = if !count_max.is_empty() {
          let parsed = count_max.parse::<usize>().map_err(|_| arity_unsupported_digit_err(count_max))?;
          Some(parsed)
        } else {
          None
        };

        if ch == SEGCOUNT_SPECIFIER_END {
          if let Some((_, next_char)) = chars.next() {
            let Arity::Range(count_min, _) = seq.seg_count else { unreachable!() };
            seq.seg_count = Arity::Range(count_min, max);

            if next_char == CHARLEN_SPECIFIER_START {
              parsing_state = ParsingState::DynCharLenMin { seq_start: *seq_start, seq, len_min: String::new() };
              continue;
            }

            if next_char == DYNAMIC_SEQUENCE_END {
              parsed_sequences.push(UrlMatcherSequence { start_pos: *seq_start, typed: TypedSequence::Dynamic(seq) });
              parsing_state = ParsingState::Literal { seq_start: index + 1, literal: String::new() };
              continue;
            }

            return Err(unexpected_char_after_segcount_close_err(ch, index));
          }

          return Err(dirname_end_after_segcount_close_err());
        }

        return Err(format!(
          "Unexpected character \"{ch}\" in Segment Count specifier upper bound.\r\n\
          Segment Count specifier upper bound can only contain numbers, and can only be followed by \
          a Segment Count specifier closing bracket – `]`."
        ));
      }
      ParsingState::DynCharLenMin { seq_start, seq, len_min } => {
        if ch.is_ascii_digit() {
          len_min.push(ch);
          continue;
        }

        if len_min.is_empty() {
          return Err(format!(
            "Unexpected character \"{ch}\" at position {index} (after Character Length specifier opening bracket – `(`).\r\n\
            Character Length specifier opening bracket must be followed by a number (exact or part of a range)."
          ));
        }

        let len_min = std::mem::take(len_min);
        let parsed = len_min.parse::<usize>().map_err(|_| arity_unsupported_digit_err(len_min))?;

        let mut seq = std::mem::take(seq);

        if ch == '.' {
          if let Some((_, next_char)) = chars.next() {
            if next_char == '.' {
              seq.char_len = Arity::Range(parsed, None);
              parsing_state = ParsingState::DynCharLenMax { seq_start: *seq_start, seq, len_max: String::new() };
              continue;
            }

            return Err(format!(
              "Unexpected character \"{ch}\" after a dot in Character Length specifier.\r\n\
              Single dot in Character Length specifier can only be followed by another dot, composing a range operator – `..`.",
            ));
          }

          return Err(String::from(
            "Directory name ends after a dot inside Character Length specifier of the dynamic sequence.\r\n\
            Both Character Length specifier and the enclosing dynamic sequence must be properly ended.",
          ));
        }

        if ch == CHARLEN_SPECIFIER_END {
          if let Some((_, next_char)) = chars.next() {
            if next_char == DYNAMIC_SEQUENCE_END {
              seq.char_len = Arity::Exact(parsed);
              parsed_sequences.push(UrlMatcherSequence { start_pos: *seq_start, typed: TypedSequence::Dynamic(seq) });
              parsing_state = ParsingState::Literal { seq_start: index + 1, literal: String::new() };
              continue;
            }

            return Err(unexpected_char_after_charlen_close_err(ch, index + 1));
          }

          return Err(dirname_end_after_charlen_close_err());
        }

        return Err(format!(
          "Unexpected character \"{ch}\" in Character Length specifier numeric bound.\r\n\
          Character Length specifier numeric bound can only contain numbers, and can only be followed by \
          either a Character Length specifier closing bracket – `)`, or range operator – `..`."
        ));
      }
      ParsingState::DynCharLenMax { seq_start, seq, len_max } => {
        if ch.is_ascii_digit() {
          len_max.push(ch);
          continue;
        }

        if ch == '.' && len_max.is_empty() {
          return Err(String::from(
            "Unexpected additional dot after Character Length range operator – `..`.\r\n\
            Character Length range operator consists of exactly two dots (`..`) followed by either a number or \
            a Character Length specifier closing bracket – `)`.",
          ));
        }

        let len_max = std::mem::take(len_max);
        let mut seq = std::mem::take(seq);

        let max: Option<usize> = if !len_max.is_empty() {
          let parsed = len_max.parse::<usize>().map_err(|_| arity_unsupported_digit_err(len_max))?;
          Some(parsed)
        } else {
          None
        };

        if ch == CHARLEN_SPECIFIER_END {
          if let Some((_, next_char)) = chars.next() {
            if next_char == DYNAMIC_SEQUENCE_END {
              let Arity::Range(len_min, _) = seq.char_len else { unreachable!() };
              seq.char_len = Arity::Range(len_min, max);

              parsed_sequences.push(UrlMatcherSequence { start_pos: *seq_start, typed: TypedSequence::Dynamic(seq) });
              parsing_state = ParsingState::Literal { seq_start: index + 1, literal: String::new() };
              continue;
            }

            return Err(unexpected_char_after_charlen_close_err(ch, index));
          }

          return Err(dirname_end_after_charlen_close_err());
        }

        return Err(format!(
          "Unexpected character \"{ch}\" in Character Length specifier upper bound.\r\n\
          Character Length specifier upper bound can only contain numbers, and can only be followed by \
          a Character Length specifier closing bracket – `)`."
        ));
      }
    }
  }

  if let ParsingState::Literal { seq_start, literal } = parsing_state {
    // Handle last literal
    if !literal.is_empty() {
      parsed_sequences.push(UrlMatcherSequence { start_pos: seq_start, typed: TypedSequence::Literal(literal) });
    }
  } else {
    // Handle invalid final parser state
    return Err(String::from(
      "A dynamic sequence is started with an opening bracket – `{`, but never closed.\r\n\
      Dynamic sequences must be properly closed.",
    ));
  };
  
  parsed_sequences.first_mut().map(|seq| {
    if let TypedSequence::Dynamic(seq) = &mut seq.typed {
      seq.is_first = true;
    }
  });
  
  parsed_sequences.last_mut().map(|seq| {
    if let TypedSequence::Dynamic(seq) = &mut seq.typed {
      seq.is_last = true;
    }
  });
  
  Ok(parsed_sequences)
}

/// Validates escape sequence and returns intact. Decoding is done in different phase.
/// This merely checks that the escape character `$` is followed by two valid hex digits.
fn decode_escape_sequence(ch1: Option<char>, ch2: Option<char>, start_pos: usize) -> Result<String, String> {
  let (Some(ch1), Some(ch2)) = (ch1, ch2) else {
    return Err(format!(
      "Incomplete dollar-encoding at position {start_pos}.\r\n\
      If you want to use a literal dollar character, encode it as \"$24\".",
    ));
  };

  if !ch1.is_ascii_hexdigit() || !ch2.is_ascii_hexdigit() {
    return Err(format!(
      "Invalid dollar-encoding \"${ch1}{ch2}\" at position {start_pos}.\r\n\
      If you want to use a literal dollar character, encode it as \"$24\".",
    ));
  }

  let seq = format!("${ch1}{ch2}");

  let decoded = dollar_encoding::decode(&seq).map_err(|_| {
    format!(
      "Invalid dollar-encoding in directory name at position {start_pos}.\r\n\
      If you want to use a literal dollar character, encode it as \"$24\".",
    )
  })?;

  Ok(decoded.into())
}

fn unexpected_special_char_err(ch: char, pos: usize) -> String {
  format!(
    "Unexpected special character \"{ch}\" at position {pos}. You might want to dollar-encode it as \"${:02X}\".",
    ch as u8
  )
}

fn unexpected_char_after_segcount_close_err(ch: char, pos: usize) -> String {
  format!(
    "Unexpected character \"{ch}\" at position {pos} after a Segment Count specifier closing bracket – `]`.\r\n\
    Segment Count specifier closing bracket can only be followed by either the dynamic sequence closing bracket – `}}`, or \
    a Segment Count specifier opening bracket – `[`"
  )
}

fn dirname_end_after_segcount_close_err() -> String {
  String::from(
    "Directory name ends after the Segment Count specifier closing bracket – `]`.\r\n\
    Did you forget to include a `}` to end the enclosing dynamic sequence?",
  )
}

fn unexpected_char_after_charlen_close_err(ch: char, pos: usize) -> String {
  format!(
    "Unexpected character \"{ch}\" at position {pos} after a Character Length specifier closing bracket – `)`.\r\n\
    Character Length specifier closing bracket can only be followed by the dynamic sequence closing bracket – `}}`."
  )
}

fn dirname_end_after_charlen_close_err() -> String {
  String::from(
    "Directory name ends after the Character Length specifier closing bracket – `)`.\r\n\
    Did you forget to include a `}` to end the enclosing dynamic sequence?",
  )
}

fn arity_unsupported_digit_err(num: String) -> String {
  format!("Dynamic sequence contains an arity specifier with unsupported digit: \"{num}\".")
}

fn validate_slot_name(slot_name: &str) -> Result<(), String> {
  if slot_name.is_empty() {
    return Err(String::from(
      "Slots must have a name.\r\n\
        If you want to match the `@` character literally, dollar-escape it as `$40`.\r\n\
        Also please note that you might want to match the URL-encoded version `%40` instead.",
    ));
  }

  if !slot_name.chars().all(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')) {
    return Err(String::from(
      "Slot names can only contain a-z, A-Z, 0-9 and _.\r\n\
        If you want to match the leading `@` character literally, dollar-escape it as `$40`.\r\n\
        Also please note that you might want to match the URL-encoded version `%40` instead.",
    ));
  }

  Ok(())
}

fn validate_custom_match_identifier(segment_name: &str) -> Result<(), String> {
  if segment_name.is_empty() {
    return Err(String::from(
      "Custom Match segments must have a name.\r\n\
        If you want to match the `~` character literally, dollar-escape it as `$7E`.",
    ));
  }

  if !segment_name.chars().all(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')) {
    return Err(String::from(
      "Custom Match segment names can only contain a-z, A-Z, 0-9 and _.\r\n\
        If you want to match the leading `~` character literally, dollar-escape it as `$7E`.",
    ));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_segment() {
    assert!(matches!(parse_segment("(foo)"), Ok(SegmentEffect::Group)));
    assert!(matches!(parse_segment("{foo[0]}"), Ok(SegmentEffect::Group)));
    assert!(matches!(parse_segment("_"), Ok(SegmentEffect::EmptySegment)));
    assert!(matches!(parse_segment("{foo[1](0)}"), Ok(SegmentEffect::EmptySegment)));
    assert!(matches!(parse_segment("~foo"), Ok(SegmentEffect::CustomMatch { identifier }) if identifier == "foo"));

    let dirname = "{myparam}";
    let SegmentEffect::UrlMatcher { sequences } = parse_segment(dirname).unwrap() else { unreachable!() };
    assert_eq!(sequences.len(), 1);
    assert!(matches!(
      sequences[0].typed,
      TypedSequence::Dynamic(DynamicSequence { seg_count: Arity::Exact(1), char_len: Arity::Range(1, None), .. })
    ));

    let dirname = "{myparam[2]}";
    let SegmentEffect::UrlMatcher { sequences } = parse_segment(dirname).unwrap() else { unreachable!() };
    assert_eq!(sequences.len(), 1);
    assert!(matches!(
      sequences[0].typed,
      TypedSequence::Dynamic(DynamicSequence { seg_count: Arity::Exact(2), char_len: Arity::Range(1, None), .. })
    ));

    let dirname = "{myparam(2)}";
    let SegmentEffect::UrlMatcher { sequences } = parse_segment(dirname).unwrap() else { unreachable!() };
    assert_eq!(sequences.len(), 1);
    assert!(matches!(
      sequences[0].typed,
      TypedSequence::Dynamic(DynamicSequence { seg_count: Arity::Exact(1), char_len: Arity::Exact(2), .. })
    ));

    let dirname = "foo-{myparam[2](3)}-bar-{myparam[0..](2..5)}-suffix";
    let SegmentEffect::UrlMatcher { sequences } = parse_segment(dirname).unwrap() else { unreachable!() };
    assert_eq!(sequences.len(), 5);
    assert!(matches!(
      (&sequences[0].typed, &sequences[2].typed, &sequences[4].typed),
      (
        TypedSequence::Literal(s0),
        TypedSequence::Literal(s2),
        TypedSequence::Literal(s4),
      ) if s0 == "foo-" && s2 == "-bar-" && s4 == "-suffix"
    ));
    assert!(matches!(
      sequences[1].typed,
      TypedSequence::Dynamic(DynamicSequence { seg_count: Arity::Exact(2), char_len: Arity::Exact(3), .. })
    ));
    assert!(matches!(
      sequences[3].typed,
      TypedSequence::Dynamic(DynamicSequence {
        seg_count: Arity::Range(0, None),
        char_len: Arity::Range(2, Some(5)),
        ..
      })
    ));
  }
}
