mod app;
mod routes_hints;

use std::path::Path;

use ::ruxy_util::fs::{get_project_dir, get_ruxy_out_dir};

pub fn build() {
  // `get_app_config()` is usable here (initialized by `build!` macro)

  let destination = get_ruxy_out_dir().join("app.rs");

  let app_contents = app::ruxy_app().unwrap_or_else(|e_tokens| e_tokens).to_string();
  std::fs::write(destination, app_contents).expect("couldn't write <out>/.ruxy/app.rs");

  let config_path = get_project_dir().join("app").join("config.rs");

  if let Some(config_path) = config_path.to_str() {
    println!("cargo::rerun-if-changed={config_path}");
  }

  routes_hints::emit_watch_hints_for_dir(Path::new("app/routes"));
}

