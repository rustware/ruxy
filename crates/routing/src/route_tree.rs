mod segment;

use std::path::Path;

pub use segment::*;

/// A complete representation of user application route tree parsed from the file system.
#[derive(Debug)]
pub struct RouteTree {
  pub segments: SegmentMap,
}

impl RouteTree {
  /// Creates a new RouteTree by parsing the filesystem at the provided path (routes directory). 
  pub fn new(routes_dir: &Path) -> Self {
    let segments = build_segments(routes_dir, routes_dir.into(), 0, None);
    RouteTree { segments }
  }
}
