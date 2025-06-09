use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::TrailingSlashConfig;

use crate::app::config::AppConfig;

pub fn wrap_router(config: &AppConfig, router: TokenStream) -> TokenStream {
  match &config.trailing_slash {
    TrailingSlashConfig::RequireAbsent => {
      // No wrapper for RequireAbsent, the "end of path" conditions only include
      // `path.is_empty()`, so the trailing slash will never match unless explicitly
      // allowed by using leaf Empty Segment or leaf `{_(0..)}` segments.
      router
    }
    TrailingSlashConfig::RedirectToRemoved => {
      quote! {
        if let Some(path) = path.strip_suffix('/') {
          if !path.is_empty() {
            return Self::redirect_to_path(&request, path);
          }
        }
        
        // RedirectToRemoved has the same behavior as RequireAbsent after
        // the user is redirected from the present slash to the absent slash.
        // No wrapping needed after the redirect.
        #router
      }
    },
    TrailingSlashConfig::RequirePresent => {
      quote! { if let Some(path) = path.strip_suffix('/') { #router } }
    }
    TrailingSlashConfig::RedirectToAdded => {
      quote! {
        let Some(path) = path.strip_suffix('/') else {
          return Self::redirect_to_added_slash(&request, path);
        }
        
        #router
      }
    }
    TrailingSlashConfig::Ignore => {
      quote! {
        let path = path.strip_suffix('/').unwrap_or(path);
        #router
      }
    }
  }
}
