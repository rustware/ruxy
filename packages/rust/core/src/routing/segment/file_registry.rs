use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

use crate::constants;
use crate::routing::segment::{
  EitherTarget, HandlerTarget, MultiTarget, RouteSegmentClientEntryExt, RenderTarget, RouteSegmentClientEntry,
  RouteSegmentRsModule,
};

struct SegmentClientFile {
  name: String,
  extension: RouteSegmentClientEntryExt,
}

#[derive(Default)]
pub struct SegmentFileRegistry {
  routes_dir: PathBuf,
  segment_rel_path: PathBuf,
  rs_module_rel_path: PathBuf,
  segment_hex: String,

  // page.rs
  route_page_rs: bool,
  // page.<js|jsx|ts|tsx|md|mdx>
  route_page_client: Option<SegmentClientFile>,
  // handler.rs
  route_handler_rs: bool,

  // not_found_page.rs
  not_found_page_rs: bool,
  // not_found_page.<js|jsx|ts|tsx|md|mdx>
  not_found_page_client: Option<SegmentClientFile>,
  // not_found_handler.rs
  not_found_handler_rs: bool,

  // error_page.rs
  error_page_rs: bool,
  // error_page.<js|jsx|ts|tsx|md|mdx>
  error_page_client: Option<SegmentClientFile>,
  // error_handler.rs
  error_handler_rs: bool,

  // layout.rs
  layout_rs: bool,
  // layout.<js|jsx|ts|tsx|md|mdx>
  layout_client: Option<SegmentClientFile>,
}

impl SegmentFileRegistry {
  pub fn new(routes_dir: &Path, segment_rel_path: &Path, hex: &str) -> Self {
    Self {
      routes_dir: routes_dir.to_path_buf(),
      segment_rel_path: segment_rel_path.to_path_buf(),
      rs_module_rel_path: PathBuf::from("routes").join(&segment_rel_path),
      segment_hex: hex.to_string(),
      ..Default::default()
    }
  }

  pub fn register(&mut self, file_name: &str) -> Result<bool, String> {
    match file_name {
      constants::SEG_FILE_PAGE_RS => self.register_route_page_rs(),
      constants::SEG_FILE_PAGE_JS => self.register_route_page_client(file_name, RouteSegmentClientEntryExt::Js),
      constants::SEG_FILE_PAGE_JSX => self.register_route_page_client(file_name, RouteSegmentClientEntryExt::Jsx),
      constants::SEG_FILE_PAGE_TS => self.register_route_page_client(file_name, RouteSegmentClientEntryExt::Ts),
      constants::SEG_FILE_PAGE_TSX => self.register_route_page_client(file_name, RouteSegmentClientEntryExt::Tsx),
      constants::SEG_FILE_PAGE_MD => self.register_route_page_client(file_name, RouteSegmentClientEntryExt::Md),
      constants::SEG_FILE_PAGE_MDX => self.register_route_page_client(file_name, RouteSegmentClientEntryExt::Mdx),
      constants::SEG_FILE_HANDLER_RS => self.register_route_handler_rs(),

      constants::SEG_FILE_NOT_FOUND_PAGE_RS => self.register_not_found_page_rs(),
      constants::SEG_FILE_NOT_FOUND_PAGE_JS => self.register_not_found_page_client(file_name, RouteSegmentClientEntryExt::Js),
      constants::SEG_FILE_NOT_FOUND_PAGE_JSX => self.register_not_found_page_client(file_name, RouteSegmentClientEntryExt::Jsx),
      constants::SEG_FILE_NOT_FOUND_PAGE_TS => self.register_not_found_page_client(file_name, RouteSegmentClientEntryExt::Ts),
      constants::SEG_FILE_NOT_FOUND_PAGE_TSX => self.register_not_found_page_client(file_name, RouteSegmentClientEntryExt::Tsx),
      constants::SEG_FILE_NOT_FOUND_PAGE_MD => self.register_not_found_page_client(file_name, RouteSegmentClientEntryExt::Md),
      constants::SEG_FILE_NOT_FOUND_PAGE_MDX => self.register_not_found_page_client(file_name, RouteSegmentClientEntryExt::Mdx),
      constants::SEG_FILE_NOT_FOUND_HANDLER_RS => self.register_not_found_handler_rs(),

      constants::SEG_FILE_ERROR_PAGE_RS => self.register_error_page_rs(),
      constants::SEG_FILE_ERROR_PAGE_JS => self.register_error_page_client(file_name, RouteSegmentClientEntryExt::Js),
      constants::SEG_FILE_ERROR_PAGE_JSX => self.register_error_page_client(file_name, RouteSegmentClientEntryExt::Jsx),
      constants::SEG_FILE_ERROR_PAGE_TS => self.register_error_page_client(file_name, RouteSegmentClientEntryExt::Ts),
      constants::SEG_FILE_ERROR_PAGE_TSX => self.register_error_page_client(file_name, RouteSegmentClientEntryExt::Tsx),
      constants::SEG_FILE_ERROR_PAGE_MD => self.register_error_page_client(file_name, RouteSegmentClientEntryExt::Md),
      constants::SEG_FILE_ERROR_PAGE_MDX => self.register_error_page_client(file_name, RouteSegmentClientEntryExt::Mdx),
      constants::SEG_FILE_ERROR_HANDLER_RS => self.register_error_handler_rs(),

      constants::SEG_FILE_LAYOUT_RS => self.register_layout_rs(),
      constants::SEG_FILE_LAYOUT_JS => self.register_layout_client(file_name, RouteSegmentClientEntryExt::Js),
      constants::SEG_FILE_LAYOUT_JSX => self.register_layout_client(file_name, RouteSegmentClientEntryExt::Jsx),
      constants::SEG_FILE_LAYOUT_TS => self.register_layout_client(file_name, RouteSegmentClientEntryExt::Ts),
      constants::SEG_FILE_LAYOUT_TSX => self.register_layout_client(file_name, RouteSegmentClientEntryExt::Tsx),
      constants::SEG_FILE_LAYOUT_MD => self.register_layout_client(file_name, RouteSegmentClientEntryExt::Md),
      constants::SEG_FILE_LAYOUT_MDX => self.register_layout_client(file_name, RouteSegmentClientEntryExt::Mdx),
      _ => Ok(false),
    }
  }

  pub fn take_route_target(&mut self) -> Option<EitherTarget> {
    if self.route_handler_rs {
      let rs_module = self.get_rs_module("route_handler", constants::SEG_FILE_HANDLER_RS);
      return Some(EitherTarget::Handler(HandlerTarget { rs_module }));
    }

    let page_client = self.route_page_client.take();

    let mut target = RenderTarget { client_entry: self.get_client_entry(page_client), rs_module: None };

    if self.route_page_rs {
      target.rs_module = Some(self.get_rs_module("route_page", constants::SEG_FILE_PAGE_RS));
    }

    if target.client_entry.is_some() || target.rs_module.is_some() {
      return Some(EitherTarget::Render(target));
    }

    None
  }

  pub fn take_not_found_target(&mut self) -> Option<EitherTarget> {
    if self.not_found_handler_rs {
      let rs_module = self.get_rs_module("not_found_handler", constants::SEG_FILE_NOT_FOUND_HANDLER_RS);
      return Some(EitherTarget::Handler(HandlerTarget { rs_module }));
    }

    let page_client = self.not_found_page_client.take();

    let mut target = RenderTarget { client_entry: self.get_client_entry(page_client), rs_module: None };

    if self.not_found_page_rs {
      target.rs_module = Some(self.get_rs_module("not_found_page", constants::SEG_FILE_NOT_FOUND_PAGE_RS));
    }

    if target.client_entry.is_some() || target.rs_module.is_some() {
      return Some(EitherTarget::Render(target));
    }

    None
  }

  pub fn take_error_target(&mut self) -> Option<MultiTarget> {
    let mut target = MultiTarget::default();

    if self.error_handler_rs {
      let rs_module = self.get_rs_module("error_handler", constants::SEG_FILE_ERROR_HANDLER_RS);
      target.handler = Some(HandlerTarget { rs_module });
    }

    let page_client = self.error_page_client.take();

    let mut render_target = RenderTarget { client_entry: self.get_client_entry(page_client), rs_module: None };

    if self.error_page_rs {
      render_target.rs_module = Some(self.get_rs_module("error_page", constants::SEG_FILE_ERROR_PAGE_RS));
    }

    if render_target.client_entry.is_some() || render_target.rs_module.is_some() {
      target.render = Some(render_target);
    }

    if target.render.is_some() || target.handler.is_some() {
      return Some(target);
    }

    None
  }

  pub fn take_layout_target(&mut self) -> Option<RenderTarget> {
    let layout_client = self.layout_client.take();

    let mut target = RenderTarget { client_entry: self.get_client_entry(layout_client), rs_module: None };

    if self.layout_rs {
      target.rs_module = Some(self.get_rs_module("layout", constants::SEG_FILE_LAYOUT_RS));
    }

    if target.client_entry.is_some() || target.rs_module.is_some() {
      return Some(target);
    }

    None
  }

  fn get_rs_module(&self, name: &str, file: &str) -> RouteSegmentRsModule {
    let module_prefix = format!("rsgmod_{}_", self.segment_hex);
    let path = &self.rs_module_rel_path.join(file);

    RouteSegmentRsModule { name: format!("{module_prefix}{name}"), path: path.to_str().unwrap_or("").to_string() }
  }

  fn get_client_entry(&self, client_file: Option<SegmentClientFile>) -> Option<RouteSegmentClientEntry> {
    let page_client = client_file?;
    let path = self.segment_rel_path.join(&page_client.name);
    Some(RouteSegmentClientEntry { name: page_client.name, path, ext: page_client.extension })
  }

  fn register_route_page_rs(&mut self) -> Result<bool, String> {
    if self.route_handler_rs {
      return Err(self.get_file_conflict_error(constants::SEG_FILE_PAGE_RS, constants::SEG_FILE_HANDLER_RS));
    }

    self.route_page_rs = true;
    Ok(true)
  }

  fn register_route_page_client(&mut self, file_name: &str, ext: RouteSegmentClientEntryExt) -> Result<bool, String> {
    if self.route_page_client.is_some() {
      return Err(self.get_client_extension_conflict_error(file_name));
    }

    if self.route_handler_rs {
      return Err(self.get_file_conflict_error(file_name, constants::SEG_FILE_HANDLER_RS));
    }

    self.route_page_client = Some(SegmentClientFile { name: file_name.to_string(), extension: ext });
    Ok(true)
  }

  fn register_route_handler_rs(&mut self) -> Result<bool, String> {
    if self.route_page_rs {
      return Err(self.get_file_conflict_error(constants::SEG_FILE_HANDLER_RS, constants::SEG_FILE_PAGE_RS));
    }

    if let Some(client) = &self.route_page_client {
      return Err(self.get_file_conflict_error(constants::SEG_FILE_HANDLER_RS, &client.name));
    }

    self.route_handler_rs = true;
    Ok(true)
  }

  fn register_not_found_page_rs(&mut self) -> Result<bool, String> {
    if self.not_found_handler_rs {
      return Err(
        self.get_file_conflict_error(constants::SEG_FILE_NOT_FOUND_PAGE_RS, constants::SEG_FILE_NOT_FOUND_HANDLER_RS),
      );
    }

    self.not_found_page_rs = true;
    Ok(true)
  }

  fn register_not_found_page_client(&mut self, file_name: &str, ext: RouteSegmentClientEntryExt) -> Result<bool, String> {
    if self.not_found_page_client.is_some() {
      return Err(self.get_client_extension_conflict_error(file_name));
    }

    if self.not_found_handler_rs {
      return Err(self.get_file_conflict_error(file_name, constants::SEG_FILE_NOT_FOUND_HANDLER_RS));
    }

    self.not_found_page_client = Some(SegmentClientFile { name: file_name.to_string(), extension: ext });
    Ok(true)
  }

  fn register_not_found_handler_rs(&mut self) -> Result<bool, String> {
    if self.not_found_page_rs {
      return Err(
        self.get_file_conflict_error(constants::SEG_FILE_NOT_FOUND_HANDLER_RS, constants::SEG_FILE_NOT_FOUND_PAGE_RS),
      );
    }

    if let Some(client) = &self.not_found_page_client {
      return Err(self.get_file_conflict_error(constants::SEG_FILE_NOT_FOUND_HANDLER_RS, &client.name));
    }

    self.not_found_handler_rs = true;
    Ok(true)
  }

  fn register_error_page_rs(&mut self) -> Result<bool, String> {
    if self.error_handler_rs {
      return Err(self.get_file_conflict_error(constants::SEG_FILE_ERROR_PAGE_RS, constants::SEG_FILE_ERROR_HANDLER_RS));
    }

    self.error_page_rs = true;
    Ok(true)
  }

  fn register_error_page_client(&mut self, file_name: &str, ext: RouteSegmentClientEntryExt) -> Result<bool, String> {
    if self.error_page_client.is_some() {
      return Err(self.get_client_extension_conflict_error(file_name));
    }

    if self.error_handler_rs {
      return Err(self.get_file_conflict_error(file_name, constants::SEG_FILE_ERROR_HANDLER_RS));
    }

    self.error_page_client = Some(SegmentClientFile { name: file_name.to_string(), extension: ext });
    Ok(true)
  }

  fn register_error_handler_rs(&mut self) -> Result<bool, String> {
    if self.error_page_rs {
      return Err(self.get_file_conflict_error(constants::SEG_FILE_ERROR_HANDLER_RS, constants::SEG_FILE_ERROR_PAGE_RS));
    }

    if let Some(client) = &self.error_page_client {
      return Err(self.get_file_conflict_error(constants::SEG_FILE_ERROR_HANDLER_RS, &client.name));
    }

    self.error_handler_rs = true;
    Ok(true)
  }

  fn register_layout_rs(&mut self) -> Result<bool, String> {
    self.layout_rs = true;
    Ok(true)
  }

  fn register_layout_client(&mut self, file_name: &str, ext: RouteSegmentClientEntryExt) -> Result<bool, String> {
    self.layout_client = Some(SegmentClientFile { name: file_name.to_string(), extension: ext });
    Ok(true)
  }

  fn get_file_conflict_error(&self, file1: &str, file2: &str) -> String {
    let prefix = self.get_segment_path_prefix_for_error();

    format!(
      "A route cannot contain both `{file1}` and `{file2}`.\r\n\
      Both files are present here:\r\n\
      {prefix}{{{file1}|{file2}}}",
    )
  }

  fn get_client_extension_conflict_error(&self, file_name: &str) -> String {
    let path_prefix = self.get_segment_path_prefix_for_error();
    let file_name_without_ext = file_name.split('.').next().unwrap_or("");

    format!(
      "A route cannot contain multiple `{file_name_without_ext}.<js|jsx|ts|tsx|md|mdx>` files.\r\n\
      Multiple `{file_name_without_ext}.*` files are present here:\r\n\
      {path_prefix}{file_name_without_ext}.*",
    )
  }

  fn get_segment_path_prefix_for_error(&self) -> String {
    let rel_path_str = self.segment_rel_path.to_str().unwrap_or("");

    match rel_path_str.is_empty() {
      true => format!("routes{MAIN_SEPARATOR_STR}"),
      false => format!("routes{MAIN_SEPARATOR_STR}{rel_path_str}{MAIN_SEPARATOR_STR}"),
    }
  }
}
