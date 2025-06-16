use crate::parse::parse_app_config;
use crate::trailing_slash::TrailingSlashConfig;

pub struct AppConfig where Self: Send + Sync {
  pub trailing_slash: TrailingSlashConfig,
}

pub static APP_CONFIG: AppConfig = parse_app_config();

pub fn get_app_config() -> &'static AppConfig {
  &APP_CONFIG
}
