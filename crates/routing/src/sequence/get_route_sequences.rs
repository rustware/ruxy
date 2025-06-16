use std::collections::HashSet;

use crate::segment::{
  Arity, DynamicSequence, RouteSegment, SegmentEffect, SegmentMap, TypedSequence, UrlMatcherSequence,
  get_route_segments,
};
use crate::sequence::RouteSequence;
use crate::sequence::get_segment_sequences::get_segment_sequences;

pub fn get_route_sequences(segments: &SegmentMap, segment: &RouteSegment) -> Result<Vec<RouteSequence>, Vec<String>> {
  let route_segments = get_route_segments(segments, segment);

  let mut errors = Vec::new();

  let mut seen_seg_count_range: Option<String> = None;
  let mut seen_param_names = HashSet::new();

  for segment in &route_segments {
    let SegmentEffect::UrlMatcher { sequences } = &segment.effect else {
      continue;
    };

    let Some(UrlMatcherSequence { typed: TypedSequence::Dynamic(sequence), .. }) = sequences.first() else {
      continue;
    };

    if seen_param_names.contains(&sequence.param_name) {
      errors.push(format!(
        "Parameter name \"{name}\" is declared for multiple Dynamic Sequences in the same route.\n\
        Each parameter name has to be unique to its route.\n\
        Full route: {full}",
        name = sequence.param_name,
        full = segment.identifier
      ));
    }

    seen_param_names.insert(&sequence.param_name);

    if matches!(sequence, DynamicSequence { seg_count: Arity::Range(_, _), .. }) {
      if let Some(seen) = seen_seg_count_range {
        errors.push(format!(
          "Sequences \"{seq1}\" and \"{seq2}\" cannot both appear in the same route.\n\
          A Dynamic Sequence with Segment Count of type Range can only appear once in a single route.\n\
          Full route: {full}",
          seq1 = seen,
          seq2 = segment.dir_name,
          full = segment.identifier
        ));
      }

      seen_seg_count_range = Some(segment.dir_name.clone());
    }
  }

  if !errors.is_empty() {
    return Err(errors);
  }

  Ok(route_segments.into_iter().flat_map(get_segment_sequences).collect())
}
