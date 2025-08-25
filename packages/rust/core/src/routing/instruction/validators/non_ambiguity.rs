use crate::routing::segment::RouteSegment;
use crate::routing::sequence::RouteSequence;

/// Takes a slice of Routes consisting with (Vec<RouteSequence>, &<route handler segment>)
pub fn validate_non_ambiguity(_routes: &[(Vec<RouteSequence>, &RouteSegment)]) -> Result<(), Vec<String>> {
  // TODO: Validate routes non-ambiguity
  
  Ok(())
}