use std::collections::{HashSet, VecDeque};

use crate::segment::{Arity, DynamicSequence, RouteSegment, SegmentMap};
use crate::sequence::get_segment_sequences::get_segment_sequences;
use crate::sequence::{MatchDirection, RouteSequence};

/// Returns the root Route Sequence containing nested Sequences as its children.
pub fn build_sequence_tree(segments: &SegmentMap) -> RouteSequence {
  let routes = segments.values().filter_map(|s| {
    s.route_handler.as_ref()?;
    Some(build_route_sequences(segments, s))
  });

  let routes: Vec<Vec<RouteSequence>> = routes.collect();

  // TODO: This is a good place to do some validation using `routes`, e.g. routes ambiguity etc.

  inflate_routes(routes)
}

fn build_route_sequences(segments: &SegmentMap, segment: &RouteSegment) -> Vec<RouteSequence> {
  let segments = build_route_segments(segments, segment);

  let sequences = segments.into_iter().flat_map(get_segment_sequences).collect();
  let sequences = split_multi_segment_sequences(sequences);
  let sequences = process_path_sequences(sequences);
  let sequences = process_segment_sequences(sequences);

  annotate_route_conclusion(sequences, segment)
}

fn build_route_segments<'a>(segments: &'a SegmentMap, segment: &'a RouteSegment) -> Vec<&'a RouteSegment> {
  let Some(parent) = segment.parent.as_ref().and_then(|id| segments.get(id)) else {
    return vec![segment];
  };

  let mut parent_segments = build_route_segments(segments, parent);
  parent_segments.push(segment);
  parent_segments
}

/// When a SegCount Range sequence is encountered, we need to flip the order of remaining
/// sequences inside the URL pathname, so the path will be matched from the end to start.
fn process_path_sequences(sequences: Vec<RouteSequence>) -> Vec<RouteSequence> {
  let mut ltr_sequences = vec![];
  let mut rtl_sequences = vec![];

  let mut url_path_direction = MatchDirection::Ltr;
  let mut target_container = &mut ltr_sequences;

  for mut sequence in sequences {
    if sequence.is_seg_count_range() {
      url_path_direction = MatchDirection::Rtl;
      target_container = &mut rtl_sequences;
    }

    sequence.url_path_direction = url_path_direction;
    target_container.push(sequence);
  }

  rtl_sequences.reverse();

  let mut result = Vec::new();

  result.extend(ltr_sequences);
  result.extend(rtl_sequences);

  result
}

/// When a CharLen Range sequence is encountered, we need to flip the order of remaining sequences
/// inside the segment, so the sequences inside the segment will be matched from the end to start.
///
/// Additionally, this function annotates the segment-start and segment-end sequences.
fn process_segment_sequences(sequences: Vec<RouteSequence>) -> Vec<RouteSequence> {
  // The result vector of re-arranged & annotated sequences
  let mut result = vec![];

  // A container for sequence slices that are to be reversed before putting into `sorted`
  let mut flipped = vec![];

  let mut sequences = sequences.into_iter().peekable();

  while let Some(mut sequence) = sequences.next() {
    if sequence.is_char_len_range() || !flipped.is_empty() {
      sequence.url_segment_direction = match sequence.url_path_direction {
        MatchDirection::Rtl => MatchDirection::Ltr,
        MatchDirection::Ltr => MatchDirection::Rtl,
      };

      flipped.push(sequence);
    } else {
      sequence.url_segment_direction = sequence.url_path_direction;
      result.push(sequence);
    }

    let is_url_segment_end = match sequences.peek() {
      Some(next) => matches!(next.matcher, RouteSequenceMatcher::Slash),
      None => true,
    };

    if is_url_segment_end {
      flipped.reverse();
      result.extend(std::mem::take(&mut flipped));
      result.last_mut().unwrap().is_segment_end = true;
    }
  }

  // Annotate segment-start and segment-end sequences
  // TODO: We might not need this (we have `concludes_segment_id` now)

  let mut seen_segment_ids = HashSet::new();

  for sequence in result.iter_mut() {
    if !seen_segment_ids.insert(&sequence.containing_segment_id) {
      sequence.is_segment_start = true;
    }

    if let RouteSequenceMatcher::Literal(literal) = &mut sequence.matcher {
      if sequence.url_path_direction == MatchDirection::Rtl && sequence.url_segment_direction == MatchDirection::Rtl {
        let _ = std::mem::replace(literal, literal.chars().rev().collect());
      }
    };
  }

  let mut seen_segment_ids = HashSet::new();

  for sequence in result.iter_mut().rev() {
    if !seen_segment_ids.insert(&sequence.containing_segment_id) {
      sequence.is_segment_end = true;
    }
  }

  result
}

/// Split every (SegCount.min > 1 && CharLen.Exact) sequence into 2 sequences separated by a slash.
/// E.g. converts `{_[n](m)}` to `{_[1](m)}` + Slash + `{_[n-1](m)}`,
/// or `{_[n..y](m)}` to `{_[1](m)}` + Slash + `{_[n-1..y](m)}`.
/// This is to allow users to write `{a}-{b[n](m)}-{c}`.
fn split_multi_segment_sequences(sequences: Vec<RouteSequence>) -> Vec<RouteSequence> {
  let mut result = vec![];

  for sequence in sequences {
    let RouteSequenceMatcher::Dynamic(dyn_seq) = &sequence.matcher else {
      result.push(sequence);
      continue;
    };

    if let DynamicSequence { seg_count: Arity::Exact(count @ 2..), char_len: Arity::Exact(char_count), .. } = dyn_seq {
      let capture_char_count = Some(char_count * count - (count - 1));
      let seq = DynamicSequence { seg_count: Arity::Exact(1), capture_char_count, ..dyn_seq.clone() };
      let seq_separated = RouteSequence { matcher: RouteSequenceMatcher::Dynamic(seq), ..sequence.clone() };

      let capture_char_count = None;
      let seq = DynamicSequence { seg_count: Arity::Exact(*count - 1), capture_char_count, ..dyn_seq.clone() };
      let seq_rest = RouteSequence { matcher: RouteSequenceMatcher::Dynamic(seq), ..sequence.clone() };

      result.push(seq_separated);
      result.push(RouteSequence { matcher: RouteSequenceMatcher::Slash, ..sequence });
      result.push(seq_rest);

      continue;
    };

    if let DynamicSequence { seg_count: Arity::Range(min @ 2.., max), char_len: Arity::Exact(_), .. } = dyn_seq {
      let split_match = Some(SplitMatch { is_separated: true });
      let seq = DynamicSequence { seg_count: Arity::Exact(1), split_match, ..dyn_seq.clone() };
      let seq_separated = RouteSequence { matcher: RouteSequenceMatcher::Dynamic(seq), ..sequence.clone() };

      let arity = Arity::Range(*min - 1, (*max).map(|max| max - 1));
      let split_match = Some(SplitMatch { is_separated: false });
      let seq = DynamicSequence { seg_count: arity, split_match, ..dyn_seq.clone() };
      let seq_rest = RouteSequence { matcher: RouteSequenceMatcher::Dynamic(seq), ..sequence.clone() };

      result.push(seq_separated);
      result.push(RouteSequence { matcher: RouteSequenceMatcher::Slash, ..sequence });
      result.push(seq_rest);

      continue;
    };

    result.push(sequence);
  }

  result
}

fn annotate_route_conclusion(mut sequences: Vec<RouteSequence>, segment: &RouteSegment) -> Vec<RouteSequence> {
  if let Some(last) = sequences.last_mut() {
    last.concludes_segment_id = Some(segment.identifier.clone());
  }

  sequences
}

/// Takes a vector of routes that are represented as a vector of sequences,
/// and creates a nested structure with common ancestors. This can also be
/// understood as a reverse effect of flattening.
fn inflate_routes(routes: Vec<Vec<RouteSequence>>) -> RouteSequence {
  let mut root = RouteSequence {
    is_segment_start: true,
    is_segment_end: true,
    containing_segment_id: "".to_string(),
    matcher: RouteSequenceMatcher::Root,
    url_path_direction: MatchDirection::Ltr,
    url_segment_direction: MatchDirection::Ltr,
    concludes_segment_id: None,
    children: vec![],
  };

  for route in routes {
    inflate_routes_recursive(&mut root, VecDeque::from(route));
  }

  root
}

fn inflate_routes_recursive(current: &mut RouteSequence, mut route: VecDeque<RouteSequence>) {
  let Some(sequence) = route.pop_front() else {
    return;
  };

  if let Some(child) = current.children.iter_mut().find(|seq| **seq == sequence) {
    // The sequence already exists in the tree, so we just pass the pointer to it for the next route sequence
    return inflate_routes_recursive(child, route);
  }

  // The sequence does not exist, so we push it to the current node's children
  current.children.push(sequence);

  // ...and pass a pointer to it for the next route sequence
  let inserted_ref = current.children.last_mut().unwrap();
  inflate_routes_recursive(inserted_ref, route);
}
