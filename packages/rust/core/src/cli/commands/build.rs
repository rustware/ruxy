use std::path::PathBuf;

use crate::routing::routary::Routary;

use crate::cli::helpers::resolve_project_directory;

pub struct CmdBuildOptions {
  pub directory: Option<PathBuf>,
}

pub fn cmd_build(mut options: CmdBuildOptions) {
  let project_dir = resolve_project_directory(options.directory.take());
  let routes_dir = project_dir.join("app/routes");
  let routes = Routary::parse(&routes_dir);
  
  println!("Routes:");
  println!("{:#?}", routes);
}
