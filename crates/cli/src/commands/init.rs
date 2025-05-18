mod get_client_language;
mod get_project_directory;
mod get_project_name;

use std::path::PathBuf;

use get_client_language::*;
use get_project_directory::*;
use get_project_name::*;

pub struct CmdInitOptions {
  pub directory: Option<PathBuf>,
  pub name: Option<String>,
  pub enable_typescript: Option<bool>,
}

#[derive(Copy, Clone)]
pub enum ClientLanguage {
  JS,
  TS,
}

pub fn cmd_init(mut options: CmdInitOptions) {
  let directory = get_project_directory(&options);
  let name = get_project_name(&mut options, &directory);
  let client_language = get_client_language(&options);
}
