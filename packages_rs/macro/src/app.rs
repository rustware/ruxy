mod config;
mod errors;
mod handler;
mod input;
mod main;
mod routes;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_config::register_app_config;
use ::ruxy_routing::route_tree::RouteTree;
use ::ruxy_util::fs::get_project_dir;

use crate::helpers::render_routes_watch;

use errors::render_errors;

pub fn ruxy_app(input: TokenStream) -> Result<TokenStream, TokenStream> {
  let input = input.try_into()?;

  register_app_config(super::config::revive_config());

  let project_dir = get_project_dir();
  let routes_dir = project_dir.join("app/routes");

  let routes = RouteTree::new(&routes_dir);

  let route_modules = routes::gen_route_modules(&routes);
  let config_module = config::gen_config_module();

  let handler_function = handler::gen_handler_function(&input, &routes);
  let main_function = main::gen_main_function();

  let errors = routes.get_compile_errors();
  let errors = render_errors(errors);

  let routes_watch = render_routes_watch();

  let main_fn_ident = Ident::new("main", Span::call_site());
  let app_mod_ident = Ident::new("app", Span::mixed_site());

  let output = quote! {
    #[path = ""]
    mod #app_mod_ident {
      #config_module
      #route_modules

      use ::ruxy::__ruxy_macro_internal as internal;

      pub(super) fn #main_fn_ident() {
        internal::register_app_config(config::config());
        
        struct App;

        impl internal::Server for App {
          #handler_function
          #main_function
        };

        <App as internal::Server>::start();

        // This will re-expand the macro on filesystem changes to the routes/ dir.
        #routes_watch
      }
    }

    use #app_mod_ident::main;

    // This will output `compile_error!(...)`, listing all collected errors.
    // This is so that macro can still expand successfully even when errors are
    // encountered, while the user can see the errors in the IDE or at build time.
    #errors
  };

  Ok(output)
}
