use std::collections::HashMap;
use std::fs;
use std::path::{MAIN_SEPARATOR_STR, PathBuf};

use rand::distr::{Alphanumeric, SampleString};

use super::SegmentMap;
use super::get_routes_dir::*;
use super::segment::{RequestHandler, RouteSegment, RouteSegmentFileModule};

/// Please reade the documentation for the `RouteSegment` struct to understand what a Route Segment is
pub fn get_segments(dir: PathBuf, depth: usize, parent_id: Option<String>) -> SegmentMap {
  let Some(dir_name) = dir.file_name() else {
    // This should never happen, since we're only ever listing absolute paths
    panic!("Leaf path segment is an entry to the parent directory");
  };

  let Some(dir_name) = dir_name.to_str() else {
    // Ignore directories with invalid characters in their name
    return HashMap::new();
  };

  let Ok(rel_path) = dir.strip_prefix(get_routes_dir()) else {
    // This should never happen, since we're only ever listing children directories
    panic!("Prefix does not match the path to the routes directory");
  };

  let Some(rel_path_str) = rel_path.to_str() else {
    // Ignore paths with invalid characters in their name
    return HashMap::new();
  };

  // Keep identifier same as relative path for now. This may change in the future.
  let identifier = rel_path_str;

  let Ok(entries) = fs::read_dir(&dir) else {
    // Ignore unreadable dirs
    return HashMap::new();
  };

  let mut module_prefix: Option<String> = None;

  let mut get_module = |name: &str, file: &str| -> RouteSegmentFileModule {
    // Only generate the module prefix once
    let prefix = module_prefix.get_or_insert_with(gen_module_prefix);
    let path = PathBuf::from("./routes").join(rel_path).join(file);

    RouteSegmentFileModule {
      name: format!("{}{}", prefix, name),
      // The values are already sanitized, `unwrap` is safe here.
      path: path.to_str().unwrap().to_string(),
    }
  };

  let mut route_handler: Option<RequestHandler> = None;
  let mut not_found_handler: Option<RequestHandler> = None;
  let mut error_handler: Option<RequestHandler> = None;
  let mut layout_module: Option<RouteSegmentFileModule> = None;

  let mut child_segments = HashMap::new();

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
      let segments = get_segments(path, depth + 1, Some(identifier.into()));
      child_segments.extend(segments);
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

          panic!(
            "A route cannot contain both `{prefix}page.rs` and `{prefix}handler.rs`.\
            Both files are present here: `routes{path}{prefix}*.rs`",
          );
        }

        let suffix = match is_custom {
          true => "handler",
          false => "page",
        };

        let module = get_module(&format!("{prefix}{suffix}"), file_name);

        let handler = match is_custom {
          true => RequestHandler::Custom { module },
          false => RequestHandler::Custom { module },
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
    // Leaf segment MUST have a Route Handler
    return HashMap::new();
  }

  let segment = RouteSegment {
    identifier: identifier.into(),
    dir_name: dir_name.into(),
    fs_rel_path: rel_path.to_path_buf(),
    fs_abs_path: dir.to_path_buf(),
    parent: parent_id,
    route_handler,
    not_found_handler,
    error_handler,
    layout_module,
    is_root,
    is_leaf,
  };

  // Rename the variable to make it clear we're basing the returned
  // map on the child segments to avoid pointless allocation.
  let mut segments = child_segments;

  segments.insert(identifier.into(), segment);

  segments
}

fn gen_module_prefix() -> String {
  // We're generating a random string as part of the prefix, so that we don't have to rely on directories
  format!("routesegment_{}_", Alphanumeric.sample_string(&mut rand::rng(), 16).to_ascii_lowercase())
}
