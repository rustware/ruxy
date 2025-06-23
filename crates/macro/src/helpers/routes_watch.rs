use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_util::fs::get_cache_dir;

/// Renders tokens that make the macro re-expand on routes/ dir changes.
pub fn render_routes_watch() -> TokenStream {
  let cache_dir = get_cache_dir();

  let routes_hash_file = cache_dir.join("ROUTES_HASH");

  match routes_hash_file.exists() {
    true => {
      let Some(routes_hash_file_str) = routes_hash_file.to_str() else {
        panic!("invalid cache dir (your build directory contains invalid characters)");
      };

      quote! { let _ = include_bytes!(#routes_hash_file_str); }
    }
    false => TokenStream::new(),
  }
}
