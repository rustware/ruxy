use proc_macro2::TokenStream;
use quote::quote;

use crate::util::fs::get_project_dir;

pub fn gen_config_module() -> TokenStream {
  let config_file = get_project_dir();
  let config_file = config_file.join("app/config.rs");

  if !config_file.is_file() {
    return quote! {
      mod config {
        pub fn config() -> ::ruxy::AppConfig {
          ::ruxy::AppConfig::default()
        }
      }
    };
  }

  let Some(path) = config_file.to_str() else {
    panic!("could not read config file path");
  };

  quote! {
    #[path = #path]
    mod config;
  }
}
