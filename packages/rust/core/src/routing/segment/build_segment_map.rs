use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use crate::routing::segment::parse_segment::parse_segment;

use super::{RouteSegment, SegmentEffect, SegmentFileRegistry, SegmentIdentifier, SegmentMap, create_segment_id};

/// Please read the documentation for the `RouteSegment` struct to understand what a Route Segment is.
/// Returns (<all nested children map>, <self identifier>)
pub fn build_segment_map(
  routes_dir: &Path,
  dir: &Path,
  depth: usize,
  parent_id: Option<String>,
) -> (SegmentMap, SegmentIdentifier) {
  let Some(dir_name) = dir.file_name() else {
    // This should never happen, since we're only ever listing absolute paths
    panic!("Leaf path segment is an entry to the parent directory");
  };

  let Some(dir_name) = dir_name.to_str() else {
    // Ignore directories with invalid characters in their name
    return (HashMap::new(), "".into());
  };

  let Ok(rel_path) = dir.strip_prefix(routes_dir) else {
    // This should never happen, since we're only ever listing children directories
    panic!("Prefix does not match the path to the routes directory");
  };

  let identifier = &create_segment_id(rel_path);

  let Ok(entries) = dir.read_dir() else {
    // Ignore unreadable dirs
    return (HashMap::new(), "".into());
  };

  let hex = gen_segment_hex(identifier);

  let mut file_registry = SegmentFileRegistry::new(routes_dir, rel_path, &hex);

  let mut child_ids = Vec::new();
  let mut child_segments = HashMap::new();
  let mut compile_errors = Vec::new();

  for entry in entries.into_iter() {
    let Ok(entry) = entry else {
      // Ignore unreadable entries
      continue;
    };

    let path = entry.path();

    if path.is_symlink() {
      // Ignore symbolic links
      // TODO: Do we want this?
      continue;
    }

    if path.is_dir() {
      if let Some(name) = path.file_name().and_then(OsStr::to_str)
        && name.starts_with('_')
        && name.len() > 1
      {
        // User's private code directory
        continue;
      }

      let (segments, id) = build_segment_map(routes_dir, &path, depth + 1, Some(identifier.into()));

      if !segments.is_empty() && !id.is_empty() {
        child_segments.extend(segments);
        child_ids.push(id);
      }

      continue;
    }

    if path.is_file() {
      let Some(file_name) = path.file_name() else {
        // We're only ever listing absolute paths, so this should never happen
        continue;
      };

      let Some(file_name) = file_name.to_str() else {
        // Skip files with invalid characters in their name
        continue;
      };

      if let Err(err) = file_registry.register(file_name) {
        compile_errors.push(err);
      }
    }
  }

  let is_root = depth == 0;
  let is_leaf = child_segments.is_empty();

  let route_target = file_registry.take_route_target();

  if is_leaf && route_target.is_none() {
    // Ignore leaf segments without a Route Target
    return (HashMap::new(), "".into());
  }

  let effect = match is_root {
    true => SegmentEffect::Group,
    false => parse_segment(dir_name).unwrap_or_else(|e| {
      compile_errors.push(e);

      // We prevent compiling when there's an error in the route tree,
      // so this will never affect runtime behavior in any way. We just
      // want to make sure the segment modules are still being output
      // by the `app!` macro, so we return a dummy value here.
      SegmentEffect::Group
    }),
  };

  let segment = RouteSegment {
    identifier: identifier.into(),
    dir_name: dir_name.into(),
    children: child_ids,
    parent: parent_id,
    compile_errors,
    route_target,
    not_found_target: file_registry.take_not_found_target(),
    error_target: file_registry.take_error_target(),
    layout_target: file_registry.take_layout_target(),
    is_root,
    is_leaf,
    effect,
    hex,
  };

  // Rename the variable to make it clear we're basing the returned
  // map on the child segments to avoid pointless allocation.
  let mut segments = child_segments;

  segments.insert(identifier.into(), segment);

  (segments, identifier.into())
}

fn gen_segment_hex(identifier: &str) -> String {
  // TODO: Maybe we can use a simple counter instead of encoding?
  //  e.g. sort all segments by identifier ASC and hex-encode its numeric index instead of identifier.
  //  We'll need to think about the implications of this, but it might be the best solution.
  // We pad each byte with a leading zero (`{:02x}`) to preserve unambiguity between bytes.
  identifier.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
