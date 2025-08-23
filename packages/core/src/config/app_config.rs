use tokio::sync::{OnceCell, SetError};

use crate::config::trailing_slash::TrailingSlashConfig;

pub struct AppConfig {
  pub trailing_slash: TrailingSlashConfig,
  pub addresses: &'static [&'static str],
  pub partytown: PartytownConfig,
}

impl Default for AppConfig {
  fn default() -> Self {
    AppConfig {
      trailing_slash: TrailingSlashConfig::default(),
      addresses: &["127.0.0.1:3000"],
      partytown: PartytownConfig::default(),
    }
  }
}

#[derive(Default)]
pub struct PartytownConfig {
  pub enabled: bool,
}

static APP_CONFIG: OnceCell<AppConfig> = OnceCell::const_new();

pub fn get_app_config() -> &'static AppConfig {
  APP_CONFIG.get().expect("app config not initialized")
}

pub fn register_app_config(app_config: AppConfig) {
  if let Err(err) = APP_CONFIG.set(app_config) {
    match err {
      SetError::AlreadyInitializedError(_) => {
        // TODO: Better logging
        eprintln!("warning: app config already initialized");
      }
      SetError::InitializingError(_) => {
        // TODO: Better logging
        eprintln!("warning: app config already being initialized");
      }
    }
  }
}
