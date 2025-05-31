mod config;
mod errors;
mod handler;
mod routes;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_routing::RouteTree;

use crate::app::config::parse_app_config;
use crate::app::errors::render_errors;
use crate::app::routes::{gen_handler_function, gen_module_declarations};
use crate::helpers::get_project_dir;

pub fn ruxy_app(config: impl Into<TokenStream>) -> proc_macro::TokenStream {
  let config = parse_app_config(config.into());

  let project_dir = get_project_dir();
  let routes_dir = project_dir.join("src/routes");
  let cache_dir = project_dir.join(".ruxy");

  let main_fn_ident = Ident::new("main", Span::call_site());

  let build_tag_file = cache_dir.join("BUILD_SCRIPT_RUN_TAG");

  let build_tag = match build_tag_file.exists() {
    true => quote! { let _ = include_bytes!("../.ruxy/BUILD_SCRIPT_RUN_TAG"); },
    false => TokenStream::new(),
  };

  let routes = RouteTree::new(&routes_dir);

  let module_declarations = gen_module_declarations(&routes);
  let handler_function = gen_handler_function(&config, &routes);

  let errors = routes.get_compile_errors();
  let errors = render_errors(errors);

  let output = quote! {
    #module_declarations

    fn #main_fn_ident() {
      // This trick will make the compiler re-expand the macro
      // when there is a filesystem change inside routes/ dir.
      #build_tag

      use ::ruxy::macro_internal as internal;

      struct App;

      impl internal::Server for App {
        #handler_function
      };

      <App as internal::Server>::start();
    }

    // This will output `compile_error!(...)`, listing all collected errors.
    // This is so that macro can still expand successfully even when errors are
    // encountered, while the user can see the errors in the IDE or at build time.
    #errors
  };

  output.into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_generates_main_function() {
    let code = ruxy_app(TokenStream::new());
    assert_eq!(code.to_string(), "fn main() {}");
  }
}
