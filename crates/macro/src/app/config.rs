use proc_macro2::TokenStream;

use ::ruxy_routing::TrailingSlashConfig;

pub struct AppConfig {
  pub trailing_slash: TrailingSlashConfig,
}

pub fn parse_app_config(_macro_input: TokenStream) -> AppConfig {
  // TODO: Parse config from macro input AND ruxy.toml
  
  AppConfig {
    trailing_slash: TrailingSlashConfig::Ignore,
  }
}
