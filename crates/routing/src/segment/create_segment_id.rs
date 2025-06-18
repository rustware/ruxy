use std::path::{Path};

/// Takes a relative path to a Route Segment from the routes/ directory
/// and returns a normalized version of that path, a.k.a. Segment ID.
pub fn create_segment_id(routes_relative_path: &Path) -> String {
  let segments = routes_relative_path.components().filter_map(|c| c.as_os_str().to_str());
  segments.collect::<Vec<&str>>().join("/")
}
