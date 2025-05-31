mod segment;

use std::path::Path;

pub use segment::*;

/// A complete representation of user application route tree parsed from the file system.
#[derive(Debug)]
pub struct RouteTree {
  pub segments: SegmentMap,
  pub root_id: Option<String>,
}

impl RouteTree {
  /// Creates a new RouteTree by parsing the filesystem at the provided path (routes directory). 
  pub fn new(routes_dir: &Path) -> Self {
    let (segments, root_id) = build_segments(routes_dir, routes_dir, 0, None);
    
    // Returned self-id is an empty string for invalid segments too, we need to check
    // the actual presence of the root segment in the HashMap. If no routes exist,
    // neither does the root segment.
    let root_id = if segments.contains_key(&root_id) { Some(root_id) } else { None };
    
    RouteTree { segments, root_id }
  }
  
  pub fn get_compile_errors(&self) -> Vec<String> {
    let errors = self.segments.values().map(|segment| {
      let path = match segment.identifier.as_str() {
        "" => "routes".to_string(),
        _ => format!("routes/{}", segment.identifier),
      };
      
      let errors = segment.compile_errors.iter().map(|e| {
        format!(
          "Invalid route segment: \"{d}\"\r\n\
          {e}\r\n\
          Full path: {p}\r\n\
          Read more about routing conventions at https://ruxy.dev/docs/routing",
          d = segment.dir_name,
          p = path
        )
      });

      errors.collect::<Vec<_>>()
    });

    errors.flatten().collect()
  }
}
