use std::collections::HashMap;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

use crate::route_tree::segment::resolve_segment_effect::resolve_segment_effect;

use super::{RequestHandler, RouteSegment, RouteSegmentFileModule, SegmentEffect, SegmentIdentifier, SegmentMap};

/// Please read the documentation for the `RouteSegment` struct to understand what a Route Segment is.
/// Returns (<all nested children map>, <self identifier>)
pub fn build_segments(
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

  let Some(rel_path_str) = rel_path.to_str() else {
    // Ignore paths with invalid characters in their name
    return (HashMap::new(), "".into());
  };

  // Keep identifier same as relative path for now. This may change in the future.
  let identifier = rel_path_str;

  let Ok(entries) = dir.read_dir() else {
    // Ignore unreadable dirs
    return (HashMap::new(), "".into());
  };

  let hex = gen_segment_hex(identifier);
  
  let module_prefix = format!("ruxy__rseg_mod_{hex}_");

  let get_module = |name: &str, file: &str| -> RouteSegmentFileModule {
    // Only generate the module prefix once
    let path = PathBuf::from("./routes").join(rel_path_str).join(file);

    RouteSegmentFileModule {
      name: format!("{}{}", module_prefix, name),
      // The values are already sanitized, `unwrap` is safe here.
      path: path.to_str().unwrap().to_string(),
    }
  };

  let mut route_handler: Option<RequestHandler> = None;
  let mut not_found_handler: Option<RequestHandler> = None;
  let mut error_handler: Option<RequestHandler> = None;
  let mut layout_module: Option<RouteSegmentFileModule> = None;

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
      continue;
    }

    if path.is_dir() {
      let (segments, id) = build_segments(routes_dir, &path, depth + 1, Some(identifier.into()));

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

      let mut set_handler_file = |prefix: &str, is_custom: bool, var: &mut Option<RequestHandler>| {
        if var.is_some() {
          let path = match rel_path_str.is_empty() {
            true => MAIN_SEPARATOR_STR,
            false => &format!("{}{}{}", MAIN_SEPARATOR_STR, rel_path_str, MAIN_SEPARATOR_STR),
          };

          compile_errors.push(format!(
            "A route cannot contain both `{prefix}page.rs` and `{prefix}handler.rs`.\r\n\
            Both files are present here: `routes{path}{prefix}*.rs`",
          ));
        }

        let suffix = match is_custom {
          true => "handler",
          false => "page",
        };

        let module = get_module(&format!("{prefix}{suffix}"), file_name);

        let handler = match is_custom {
          true => RequestHandler::Custom { module },
          false => RequestHandler::Page { module },
        };

        *var = Some(handler);
      };

      match file_name {
        "page.rs" => set_handler_file("", false, &mut route_handler),
        "handler.rs" => set_handler_file("", true, &mut route_handler),
        "not_found_page.rs" => set_handler_file("not_found_", false, &mut not_found_handler),
        "not_found_handler.rs" => set_handler_file("not_found_", true, &mut not_found_handler),
        "error_page.rs" => set_handler_file("error_", false, &mut error_handler),
        "error_handler.rs" => set_handler_file("error_", true, &mut error_handler),
        "layout.rs" => layout_module = Some(get_module("layout", "layout.rs")),
        _ => {}
      }
    }
  }

  let is_root = depth == 0;
  let is_leaf = child_segments.is_empty();

  if is_leaf && route_handler.is_none() {
    // Leaf segment MUST have a Route Handler, otherwise is ignored
    return (HashMap::new(), "".into());
  }
  
  // TODO: Add rules:
  //  1. Multiple dynamic segments with range arity at the same level MUST NOT overlap ([2..4], [5..7] is valid, [2..4], [4..7] is invalid)
  //  2. Multiple dynamic segments with exact arity MUST NOT overlap ([2], [3] is valid, [2], [2] is invalid)
  //  3. Non-dynamic routes cannot overlap (my/(group)/route, my/route matches to the same URL and is invalid)
  //  4. Segment with page.rs MUST contain `page.(j|t)sx?`.

  let effect = match is_root {
    true => SegmentEffect::Group,
    false => resolve_segment_effect(dir_name).unwrap_or_else(|e| {
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
    route_handler,
    not_found_handler,
    error_handler,
    layout_module,
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
  identifier.as_bytes().iter().map(|b| format!("{:02x}", b)).collect()
}
