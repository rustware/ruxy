use ruxy_config::{AppConfig, TrailingSlashConfig};

// While executing proc macros, only a subset of config
// is available, as the config needs to be parsed from
// the `cfg` attributes set by the build script in the
// user's crate.
pub fn revive_config() -> AppConfig {
  AppConfig {
    trailing_slash: get_trailing_slash_config(),
    ..Default::default()
  }
}

fn get_trailing_slash_config() -> TrailingSlashConfig {
  #[cfg(ruxy_tscfg = "0")]
  return TrailingSlashConfig::RequireAbsent;
  #[cfg(ruxy_tscfg = "1")]
  return TrailingSlashConfig::RedirectToRemoved;
  #[cfg(ruxy_tscfg = "2")]
  return TrailingSlashConfig::RequirePresent;
  #[cfg(ruxy_tscfg = "3")]
  return TrailingSlashConfig::RedirectToAdded;
  #[cfg(ruxy_tscfg = "4")]
  return TrailingSlashConfig::Ignore;
  #[cfg(not(ruxy_tscfg))] // default
  TrailingSlashConfig::RedirectToRemoved
}
