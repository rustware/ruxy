use crate::segment::RouteSegment;
use crate::sequence::RouteSequence;

/// Takes a slice of Routes consisting with (Vec<RouteSequence>, &<route handler segment>)
pub fn validate_non_ambiguity(routes: &[(Vec<RouteSequence>, &RouteSegment)]) -> Result<(), Vec<String>> {
  // TODO: Validate non-ambiguity of routes
  
  Ok(())
}