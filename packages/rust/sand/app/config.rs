//  This is your application's configuration file.
//
//  It's executed by Ruxy during both build time and runtime.
//  If you use any external crates, be sure to include them
//  in both [dependencies] and [build-dependencies] sections
//  of your Cargo.toml.

use ruxy::{AppConfig, TrailingSlashConfig};

pub fn config() -> AppConfig {
  AppConfig {
    trailing_slash: TrailingSlashConfig::RedirectToRemoved,
    ..AppConfig::default()
  }
}
