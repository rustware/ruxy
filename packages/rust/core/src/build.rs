mod app;
mod build_config;
pub mod client;
mod routes_hints;

use std::path::MAIN_SEPARATOR;

use crate::constants::{APP_DIR, CONFIG_FILE, DOT_RUXY_DIR, GENERATED_APP_RS_FILE, ROUTES_DIR};
use crate::routing::routary::Routary;
use crate::util::fs::{get_project_dir, get_ruxy_out_dir};

use build_config::BuildMode;

pub use build_config::BuildConfig;

pub fn build(config: BuildConfig) {
  // `get_app_config()` is usable here too (initialized by `build!` macro)

  let project_dir = get_project_dir();
  let routes_dir = project_dir.join(APP_DIR).join(ROUTES_DIR);

  eprintln!("[ruxy] parsing routes");
  let routary = Routary::parse(&routes_dir);

  // Generate the Rust application (<out>/.ruxy/app.rs)

  println!("[ruxy] building rust application");
  let app_rs_destination = get_ruxy_out_dir().join(GENERATED_APP_RS_FILE);
  let app_rs_contents = app::ruxy_app(&routary, &config).unwrap_or_else(|e_tokens| e_tokens).to_string();

  if std::fs::write(app_rs_destination, app_rs_contents).is_err() {
    panic!("couldn't write <out>{MAIN_SEPARATOR}{DOT_RUXY_DIR}{MAIN_SEPARATOR}{GENERATED_APP_RS_FILE}")
  }

  // Build the client application (in Production build mode)

  if let BuildMode::Production = config.mode {
    println!("[ruxy] building client");
    client::build_all(&config, &routary);
  }

  // Emit watch hints for routes and special files

  println!("[ruxy] emitting watch hints");

  let config_path = project_dir.join(APP_DIR).join(CONFIG_FILE);

  if let Some(config_path) = config_path.to_str() {
    println!("cargo::rerun-if-changed={config_path}");
  }

  routes_hints::emit_watch_hints_for_dir(&routes_dir);
}
