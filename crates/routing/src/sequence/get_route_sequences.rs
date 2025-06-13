use crate::segment::{RouteSegment, SegmentMap, get_route_segments};
use crate::sequence::RouteSequence;
use crate::sequence::get_segment_sequences::get_segment_sequences;

pub fn get_route_sequences(segments: &SegmentMap, segment: &RouteSegment) -> Vec<RouteSequence> {
  let segments = get_route_segments(segments, segment);
  segments.into_iter().flat_map(get_segment_sequences).collect()
}
