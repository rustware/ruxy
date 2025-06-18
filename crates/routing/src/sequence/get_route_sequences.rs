use std::collections::HashSet;

use crate::segment::{
  Arity, DynamicSequence, RouteSegment, SegmentEffect, SegmentMap, TypedSequence, UrlMatcherSequence,
  get_route_segments,
};
use crate::sequence::RouteSequence;
use crate::sequence::get_segment_sequences::get_segment_sequences;

pub fn get_route_sequences(segments: &SegmentMap, segment: &RouteSegment) -> Result<Vec<RouteSequence>, Vec<String>> {
  let route_segments = get_route_segments(segments, segment);

  validate_route(&route_segments)?;

  Ok(route_segments.into_iter().flat_map(get_segment_sequences).collect())
}

/// Validates param uniqueness, and no more than 1 SegCount:Range in a route.
fn validate_route(route_segments: &Vec<&RouteSegment>) -> Result<(), Vec<String>> {
  let mut errors = vec![];

  let mut seen_seg_count_range: Option<String> = None;
  let mut seen_param_names = HashSet::new();
  let mut reported_param_names = HashSet::new();

  for segment in route_segments {
    let SegmentEffect::UrlMatcher { sequences } = &segment.effect else {
      continue;
    };

    let Some(UrlMatcherSequence { typed: TypedSequence::Dynamic(sequence), .. }) = sequences.first() else {
      continue;
    };

    if !seen_param_names.insert(&sequence.param_name) && reported_param_names.insert(&sequence.param_name) {
      errors.push(format!(
        "Parameter names must be unique within its route.\n\
        Parameter name \"{name}\" is declared multiple times in the same route:\n\
        {full}",
        name = sequence.param_name,
        full = segment.identifier
      ));
    }

    if matches!(sequence, DynamicSequence { seg_count: Arity::Range(_, _), .. }) {
      if let Some(seen) = seen_seg_count_range {
        errors.push(format!(
          "A Dynamic Sequence with Segment Count of type Range can only be defined once in a single route.\n\
          Sequences \"{seq1}\" and \"{seq2}\" both define a Segment Count of type Range in the same route:\n\
          {full}",
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
  
  Ok(())
}
