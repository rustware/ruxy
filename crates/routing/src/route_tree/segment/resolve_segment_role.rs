use crate::{DynamicSequenceArity, SegmentRole, UrlMatcherSequence};

const SLOT_START: &str = "@";
const DYNAMIC_SEQUENCE_START: char = '{';
const DYNAMIC_SEQUENCE_END: char = '}';
const ARITY_SPECIFIER_START: char = '[';
const ARITY_SPECIFIER_END: char = ']';
const ROUTE_GROUP_START: char = '(';
const ROUTE_GROUP_END: char = ')';

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
/// `(name)`        Route group (ignored)
///                 This is a shorthand for `{name[0]}`
///
/// `@name`         Named slot
///
/// Forbidden characters: https://stackoverflow.com/a/31976060
pub fn resolve_segment_role(dir_name: &str) -> SegmentRole {
  // Named Slots for Parallel Routes (`@my_slot`)
  if let Some(slot_name) = dir_name.strip_prefix(SLOT_START) {
    return SegmentRole::Slot { name: slot_name.to_owned() };
  }
  
  // Route Groups – shorthand form (`(group)`)
  if dir_name.starts_with(ROUTE_GROUP_START) && dir_name.ends_with(ROUTE_GROUP_END) {
    return SegmentRole::PassThrough;
  }

  let sequences = resolve_url_matcher_sequences(dir_name);
  
  for sequence in sequences.iter() {
    // Presence of any sequence with exactly 0 arity marks whole segment a passthrough segment
    if let UrlMatcherSequence::Dynamic { arity: DynamicSequenceArity::Exact(0), .. } = sequence {
      return SegmentRole::PassThrough;
    }
  }
  
  SegmentRole::UrlMatcher { sequences }
}

fn resolve_url_matcher_sequences(dir_name: &str) -> Vec<UrlMatcherSequence> {
  enum ParsingPhase {
    Prefix,
    Dynamic,
    Suffix,
  }

  let mut parsing_phase = ParsingPhase::Prefix;

  let mut prefix = String::new();
  let mut dynamic = String::new();
  let mut suffix = String::new();

  for ch in dir_name.chars() {
    match parsing_phase {
      ParsingPhase::Prefix => {
        if ch == DYNAMIC_SEQUENCE_START {
          parsing_phase = ParsingPhase::Dynamic;
          continue;
        }

        prefix.push(ch);
      }
      ParsingPhase::Dynamic => {
        if ch == DYNAMIC_SEQUENCE_END {
          parsing_phase = ParsingPhase::Suffix;
          continue;
        }

        dynamic.push(ch);
      }
      ParsingPhase::Suffix => suffix.push(ch),
    }
  }

  match parsing_phase {
    ParsingPhase::Prefix => {
      // Whole directory name is a single literal sequence
      vec![UrlMatcherSequence::Literal(dir_name.into())]
    }
    ParsingPhase::Dynamic => {
      // The dynamic sequence was not ended, hence it's not a dynamic sequence.
      // Whole directory name is thus a single literal sequence.
      vec![UrlMatcherSequence::Literal(dir_name.into())]
    }
    ParsingPhase::Suffix => {
      match parse_dynamic_sequence(dynamic) {
        None => {
          // If there's anything wrong with the dynamic sequence, it's considered
          // a literal and thus whole directory name is a single literal sequence.
          vec![UrlMatcherSequence::Literal(dir_name.into())]
        }
        Some(sequence) => {
          // The dynamic sequence is valid and was successfuly parsed.

          let mut sequences = Vec::new();

          if !prefix.is_empty() {
            sequences.push(UrlMatcherSequence::Literal(prefix));
          }

          sequences.push(sequence);

          if !suffix.is_empty() {
            sequences.push(UrlMatcherSequence::Literal(suffix));
          }

          sequences
        }
      }
    }
  }
}

fn parse_dynamic_sequence(dynamic: String) -> Option<UrlMatcherSequence> {
  enum ParsingPhase {
    VarName,
    ArityLowerBound,
    ArityBoundSeparator,
    ArityUpperBound,
    ArityClosed,
  }

  let mut parsing_phase = ParsingPhase::VarName;

  let mut var_name = String::new();
  let mut arity_is_exact_number = false;
  let mut arity_lower_bound = String::new();
  let mut arity_upper_bound = String::new();

  for ch in dynamic.chars() {
    match parsing_phase {
      ParsingPhase::VarName => {
        // Only valid Rust identifiers are considered valid var names
        if matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
          var_name.push(ch);
          continue;
        }

        if ch == ARITY_SPECIFIER_START {
          parsing_phase = ParsingPhase::ArityLowerBound;
          continue;
        }

        return None;
      }
      ParsingPhase::ArityLowerBound => {
        if ch.is_ascii_digit() {
          arity_lower_bound.push(ch);
          continue;
        }

        if ch == ARITY_SPECIFIER_END {
          // End parsing, we have an exact number (`{var[n]}`)
          arity_is_exact_number = true;
          parsing_phase = ParsingPhase::ArityClosed;
          continue;
        }

        if ch == '.' {
          parsing_phase = ParsingPhase::ArityBoundSeparator;
          continue;
        }

        return None;
      }
      ParsingPhase::ArityBoundSeparator => {
        if ch == '.' {
          parsing_phase = ParsingPhase::ArityUpperBound;
          continue;
        }

        return None;
      }
      ParsingPhase::ArityUpperBound => {
        if ch.is_ascii_digit() {
          arity_upper_bound.push(ch);
          continue;
        }

        if ch == ARITY_SPECIFIER_END {
          parsing_phase = ParsingPhase::ArityClosed;
          continue;
        }
      }
      ParsingPhase::ArityClosed => {
        // There are no more characters expected,
        // otherwise the sequence is invalid
        return None;
      }
    }
  }

  if var_name.is_empty() {
    return None;
  }

  let sequence = match parsing_phase {
    ParsingPhase::VarName => {
      // Default arity is exactly 1 if the specifier is missing. This is the implementation
      // of the alias `{var}` -> `{var[1]}`.
      UrlMatcherSequence::Dynamic { var_name, arity: DynamicSequenceArity::Exact(1) }
    }
    ParsingPhase::ArityClosed => {
      if arity_lower_bound.is_empty() {
        // [..n] (invalid)
        return None;
      }
      
      let arity = 'arity: {
        let lower_bound = arity_lower_bound.parse::<usize>().ok()?;

        if arity_is_exact_number {
          // [n]
          break 'arity DynamicSequenceArity::Exact(lower_bound);
        }

        if arity_upper_bound.is_empty() {
          // [n..]
          break 'arity DynamicSequenceArity::Range(lower_bound, None);
        }

        let upper_bound = arity_upper_bound.parse::<usize>().ok()?;

        if upper_bound < lower_bound {
          return None;
        }

        if upper_bound == lower_bound {
          // [n..n] -> [n]
          break 'arity DynamicSequenceArity::Exact(lower_bound);
        }

        // [n..m]
        DynamicSequenceArity::Range(lower_bound, Some(upper_bound))
      };
      
      UrlMatcherSequence::Dynamic { var_name, arity }
    }
    _ => {
      // Ending in any other phase is invalid
      return None;
    }
  };
  
  Some(sequence)
}
