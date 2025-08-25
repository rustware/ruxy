use std::path::PathBuf;

use ::ruxy_core::routing::segment::create_segment_id;
use ::ruxy_core::util::fs::get_project_dir;

pub struct RouteFile {
  /// An absolute path to the file on the local file system.
  pub path: PathBuf,
  /// The ID of the Route Segment this file belongs to.
  pub segment_id: String,
  /// The type of the route file.
  pub file_type: RouteFileType,
}

pub enum RouteFileType {
  Page,
  Handler,
  NotFoundPage,
  NotFoundHandler,
  ErrorPage,
  ErrorHandler,
  Layout,
  Matcher,
  Unknown,
}

impl RouteFileType {
  pub fn is_page(&self) -> bool {
    matches!(self, RouteFileType::Page | RouteFileType::NotFoundPage | RouteFileType::ErrorPage)
  }

  pub fn is_handler(&self) -> bool {
    matches!(self, RouteFileType::Handler | RouteFileType::NotFoundHandler | RouteFileType::ErrorHandler)
  }

  pub fn is_layout(&self) -> bool {
    matches!(self, RouteFileType::Layout)
  }

  pub fn is_matcher(&self) -> bool {
    matches!(self, RouteFileType::Matcher)
  }

  fn from_file_name(file_name: &str) -> RouteFileType {
    match file_name {
      "page.rs" => RouteFileType::Page,
      "handler.rs" => RouteFileType::Handler,
      "not_found_page.rs" => RouteFileType::NotFoundPage,
      "not_found_handler.rs" => RouteFileType::NotFoundHandler,
      "error_page.rs" => RouteFileType::ErrorPage,
      "error_handler.rs" => RouteFileType::ErrorHandler,
      "layout.rs" => RouteFileType::Layout,
      "match.rs" => RouteFileType::Matcher,
      _ => RouteFileType::Unknown,
    }
  }
}

pub fn get_route_file() -> RouteFile {
  let project_dir = get_project_dir();
  let routes_dir = project_dir.join("app/routes");
  let span = proc_macro::Span::call_site();

  let Some(file) = span.local_file() else {
    panic!("cannot get the macro call site file");
  };

  let Ok(cwd) = std::env::current_dir() else {
    panic!("cannot get current working directory");
  };

  let file = cwd.join(file);
  let path = file.to_path_buf();
  
  let Ok(rel_path) = file.strip_prefix(routes_dir) else {
    panic!(
      "called outside of routes directory.\n\
      Please only call this macro inside of the routes directory of your project."
    );
  };

  let segment_path = rel_path.parent().unwrap();
  let file_name = rel_path.file_name().unwrap();

  let Some(file_name) = file_name.to_str() else {
    panic!("Invalid file name: {}", file_name.display());
  };

  let file_type = RouteFileType::from_file_name(file_name);

  RouteFile { path, segment_id: create_segment_id(segment_path), file_type }
}
