use proc_macro2::TokenStream;

use ::ruxy_routing::TrailingSlashConfig;

pub struct AppConfig {
  pub trailing_slash: TrailingSlashConfig,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      trailing_slash: TrailingSlashConfig::RedirectToRemoved,
    }
  }
}

pub fn parse_app_config(_macro_input: TokenStream) -> AppConfig {
  let config: AppConfig = Default::default();

  // TODO: Parse config from macro input AND ruxy.toml and update `config`

  config
}
