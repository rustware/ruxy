#![allow(unexpected_cfgs)]

use proc_macro2::TokenStream;
use quote::quote;

use crate::config::{get_app_config, TrailingSlashConfig};

pub fn wrap_router(router: TokenStream) -> TokenStream {
  match get_app_config().trailing_slash {
    TrailingSlashConfig::RequireAbsent => {
      // No wrapper for RequireAbsent, the "end of path" conditions only include
      // `path.is_empty()`, so the trailing slash will never match unless explicitly
      // allowed by using leaf Empty Segment or leaf `{_(0..)}` segments.
      router
    }
    TrailingSlashConfig::RedirectToRemoved => {
      quote! {
        if path.ends_with('/') && path.len() > 1 {
          return Self::redirect_to_path(path.trim_end_matches('/'));
        }
  
        // RedirectToRemoved has the same behavior as RequireAbsent after
        // the user is redirected from the present slash to the absent slash.
        // No wrapping needed after the redirect.
        #router
      }
    }
    TrailingSlashConfig::RequirePresent => {
      quote! { if let Some(path) = path.strip_suffix('/') { #router } }
    }
    TrailingSlashConfig::RedirectToAdded => {
      quote! {
        let Some(path) = path.strip_suffix('/') else {
          return Self::redirect_to_added_slash(path);
        };
  
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
