use std::collections::VecDeque;

use crate::{
  MatchDirection, RouteSegment, RouteSequence, RouteSequenceMatcher, SegmentEffect, SegmentMap, TypedSequence,
};

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
  let sequences = segments.into_iter().flat_map(extract_segment_sequences).collect();

  flip_rtl_sequences(sequences)
}

fn build_route_segments<'a>(segments: &'a SegmentMap, segment: &'a RouteSegment) -> Vec<&'a RouteSegment> {
  let Some(parent) = segment.parent.as_ref().and_then(|id| segments.get(id)) else {
    return vec![segment];
  };

  let mut parent_segments = build_route_segments(segments, parent);
  parent_segments.push(segment);
  parent_segments
}

fn extract_segment_sequences(segment: &RouteSegment) -> Vec<RouteSequence> {
  let base_sequence = RouteSequence {
    containing_segment_id: segment.identifier.clone(),
    matcher: RouteSequenceMatcher::None,
    direction: MatchDirection::Ltr,
    children: vec![],
  };

  match &segment.effect {
    SegmentEffect::EmptySegment => {
      vec![RouteSequence { matcher: RouteSequenceMatcher::None, ..base_sequence }]
    }
    SegmentEffect::CustomMatch { .. } => {
      vec![RouteSequence { matcher: RouteSequenceMatcher::Custom, ..base_sequence }]
    }
    SegmentEffect::UrlMatcher { sequences: url_matcher_sequences } => {
      let mut sequences = vec![];

      for (index, seq) in url_matcher_sequences.iter().enumerate() {
        match &seq.typed {
          TypedSequence::Literal(literal) => {
            if index == 0 {
              sequences.push(RouteSequence { matcher: RouteSequenceMatcher::Slash, ..base_sequence.clone() });
            }

            sequences
              .push(RouteSequence { matcher: RouteSequenceMatcher::Literal(literal.clone()), ..base_sequence.clone() });
          }
          TypedSequence::Dynamic(seq) => {
            if index == 0 && seq.seg_count.get_min() > 0 {
              sequences.push(RouteSequence { matcher: RouteSequenceMatcher::Slash, ..base_sequence.clone() });
            }

            sequences
              .push(RouteSequence { matcher: RouteSequenceMatcher::Dynamic(seq.clone()), ..base_sequence.clone() });
          }
        }
      }

      sequences
    }
    _ => vec![],
  }
}

/// When a SegCount Range sequence is encountered, we need to flip the path
/// from that point onwards, so the URL will be matched from the end to start.
fn flip_rtl_sequences(sequences: Vec<RouteSequence>) -> Vec<RouteSequence> {
  let mut ltr_sequences = vec![];
  let mut rtl_sequences = vec![];

  let mut direction = MatchDirection::Ltr;
  let mut target = &mut ltr_sequences;

  for mut sequence in sequences {
    if sequence.is_seg_count_range() {
      direction = MatchDirection::Rtl;
      target = &mut rtl_sequences;
    }

    sequence.direction = direction;
    target.push(sequence);
  }

  rtl_sequences.reverse();

  let mut result = Vec::new();

  result.extend(ltr_sequences);
  result.extend(rtl_sequences);

  result
}

/// Takes a vector of routes that are represented as a vector of sequences,
/// and creates a nested structure with common ancestors. This can also be
/// understood as a reverse effect of flattening.
fn inflate_routes(routes: Vec<Vec<RouteSequence>>) -> RouteSequence {
  let mut root = RouteSequence {
    containing_segment_id: "".to_string(),
    matcher: RouteSequenceMatcher::None,
    direction: MatchDirection::Ltr,
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
