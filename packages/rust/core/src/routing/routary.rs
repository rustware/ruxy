#[cfg(test)]
mod test;

use std::path::Path;

use crate::routing::instruction::MatchInstruction;
use crate::routing::instruction::create_instructions::create_instructions;
use crate::routing::segment::{RouteSegment, SegmentMap, build_segment_map};

/// An internal representation of user application routes parsed from the file system.
#[derive(Debug)]
pub struct Routary {
  pub segment_map: SegmentMap,
  pub root_segment_id: Option<String>,
  pub root_match_instruction: MatchInstruction,
  pub routes_compile_errors: Vec<String>,
}

impl Routary {
  /// Creates a Routary instance by parsing the filesystem at the provided path (routes directory).
  pub fn parse(routes_dir: &Path) -> Self {
    let (segment_map, root_segment_id) = build_segment_map(routes_dir, routes_dir, 0, None);

    // Returned self-id is an empty string for invalid segments too, we need to check
    // the actual presence of the root segment in the HashMap. If no routes exist,
    // neither does the root segment.
    let root_segment_id = if segment_map.contains_key(&root_segment_id) { Some(root_segment_id) } else { None };

    let (root_match_instruction, routes_compile_errors) = match create_instructions(&segment_map) {
      Ok(instructions) => (instructions, vec![]),
      Err(errors) => (Default::default(), errors),
    };

    Routary { segment_map, root_segment_id, root_match_instruction, routes_compile_errors }
  }

  pub fn get_compile_errors(&self) -> Vec<String> {
    let mut errors = vec![];

    for segment in self.segment_map.values() {
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
    let Some(root_id) = &self.root_segment_id else {
      return None;
    };

    self.segment_map.get(root_id)
  }
}
