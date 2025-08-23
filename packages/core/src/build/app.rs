mod errors;
mod handler;
mod input;
mod main;
mod routes;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::util::fs::get_project_dir;

use crate::config::gen_config_module;
use crate::routing::route_tree::RouteTree;

use errors::render_errors;
use routes::gen_route_modules;

pub fn ruxy_app() -> Result<TokenStream, TokenStream> {
  let project_dir = get_project_dir();
  let routes_dir = project_dir.join("app/routes");

  let routes = RouteTree::new(&routes_dir);

  let route_modules = gen_route_modules(&routes);
  let config_module = gen_config_module();

  let handler_function = handler::gen_handler_function(&routes);
  let main_function = main::gen_main_function();

  let errors = routes.get_compile_errors();
  let errors = render_errors(errors);

  let main_fn_ident = Ident::new("main", Span::call_site());

  let output = quote! {
    #config_module
    #route_modules

    use ::ruxy::__ruxy_macro_internal as internal;

    pub(super) fn #main_fn_ident() {
      internal::register_app_config(config::config());

      struct App;

      impl internal::Server for App {
        #handler_function
        #main_function
      }

      <App as internal::Server>::start();
    }

    // This will output `compile_error!(...)`, listing all collected errors.
    // This is so that macro can still expand successfully even when errors are
    // encountered, while the user can see the errors in the IDE or at build time.
    #errors
  };

  Ok(output)
}
