#[cfg(test)]
mod test;

use std::path::Path;

use crate::instruction::create_instructions::create_instructions;
use crate::instruction::MatchInstruction;
use crate::segment::*;

/// A complete representation of user application route tree parsed from the file system.
#[derive(Debug)]
pub struct RouteTree {
  pub root_id: Option<String>,
  pub segments: SegmentMap,
  pub root_instruction: MatchInstruction,
  pub routes_compile_errors: Vec<String>,
}

impl RouteTree {
  /// Creates a new RouteTree by parsing the filesystem at the provided path (routes directory).
  pub fn new(routes_dir: &Path) -> Self {
    let (segments, root_id) = build_segment_map(routes_dir, routes_dir, 0, None);

    // Returned self-id is an empty string for invalid segments too, we need to check
    // the actual presence of the root segment in the HashMap. If no routes exist,
    // neither does the root segment.
    let root_id = if segments.contains_key(&root_id) { Some(root_id) } else { None };

    let (root_instruction, routes_compile_errors) = match create_instructions(&segments) {
      Ok(instructions) => (instructions, vec![]),
      Err(errors) => (Default::default(), errors),
    };
    
    RouteTree { segments, root_id, root_instruction, routes_compile_errors }
  }

  pub fn get_compile_errors(&self) -> Vec<String> {
    let mut errors = vec![];
    
    for segment in self.segments.values() {
      let path = match segment.identifier.as_str() {
        "" => "routes".to_string(),
        _ => format!("routes/{}", segment.identifier),
      };

      let segment_errors = segment.compile_errors.iter().map(|e| {
        format!(
          "Invalid route segment: \"{dir_name}\"\n\
          {e}\n\
          Full path: {path}\n\
          Read more about routing conventions at https://ruxy.dev/docs/routing",
          dir_name = segment.dir_name,
        )
      });

      errors.extend(segment_errors);
    }
    
    errors.extend(self.routes_compile_errors.clone());
    
    errors
  }

  pub fn get_root_segment(&self) -> Option<&RouteSegment> {
    let Some(root_id) = &self.root_id else {
      return None;
    };

    self.segments.get(root_id)
  }
}
