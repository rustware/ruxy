use crate::routing::segment::{RouteSegment, SegmentMap};

pub fn get_route_segments<'a>(segments: &'a SegmentMap, segment: &'a RouteSegment) -> Vec<&'a RouteSegment> {
  let Some(parent) = segment.parent.as_ref().and_then(|id| segments.get(id)) else {
    return vec![segment];
  };

  let mut parent_segments = get_route_segments(segments, parent);
  parent_segments.push(segment);
  parent_segments
}
