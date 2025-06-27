use crate::{AppConfig, TrailingSlashConfig};

pub(crate) const fn parse_app_config() -> AppConfig {
  // TODO: Generate config from ruxy.toml with a proc macro
  AppConfig { trailing_slash: TrailingSlashConfig::RedirectToRemoved }
}
