use std::path::PathBuf;

use ::ruxy_routing::RouteTree;

use crate::helpers::resolve_project_directory;

pub struct CmdBuildOptions {
  pub directory: Option<PathBuf>,
}

pub fn cmd_build(mut options: CmdBuildOptions) {
  let project_dir = resolve_project_directory(options.directory.take());
  let routes_dir = project_dir.join("src/routes");
  let routes = RouteTree::new(&routes_dir);
  
  println!("Routes:");
  println!("{:#?}", routes);
}
