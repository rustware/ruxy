mod config;
mod errors;
mod handler;
mod routes;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_routing::route_tree::RouteTree;
use ::ruxy_util::fs::get_project_dir;

use crate::app::config::parse_macro_config;
use crate::app::errors::render_errors;
use crate::helpers::render_routes_watch;

pub fn ruxy_app(input: impl Into<TokenStream>) -> proc_macro::TokenStream {
  let config = parse_macro_config(input.into());

  let project_dir = get_project_dir();
  let routes_dir = project_dir.join("src/routes");
  
  let routes = RouteTree::new(&routes_dir);

  let module_declarations = routes::gen_module_declarations(&routes);
  let handler_function = handler::gen_handler_function(&config, &routes);
  
  let errors = routes.get_compile_errors();
  let errors = render_errors(errors);

  let routes_watch = render_routes_watch();

  let main_fn_ident = Ident::new("main", Span::call_site());
  let app_mod_ident = Ident::new("app", Span::mixed_site());

  let output = quote! {
    #[path = ""]
    mod #app_mod_ident {
      #module_declarations

      use ::ruxy::__ruxy_macro_internal as internal;

      pub(super) fn #main_fn_ident() {
        struct App;

        impl internal::Server for App {
          #handler_function
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

  output.into()
}
