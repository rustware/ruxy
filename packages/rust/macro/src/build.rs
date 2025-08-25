mod input;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_core::config::gen_config_module;

use crate::build::input::BuildMacroInput;

pub fn ruxy_build(input: TokenStream) -> Result<TokenStream, TokenStream> {
  let _input: BuildMacroInput = input.try_into()?;

  let config_module = gen_config_module();

  let main_fn_ident = Ident::new("main", Span::call_site());
  let build_mod_ident = Ident::new("ruxy_build", Span::mixed_site());

  let output = quote! {
    mod #build_mod_ident {
      #config_module

      use ::ruxy::__ruxy_macro_internal as internal;

      pub(super) fn #main_fn_ident() {
        internal::register_app_config(config::config());
        internal::build(internal::BuildConfig::parse());
      }
    }

    use #build_mod_ident::#main_fn_ident;
  };

  Ok(output)
}
