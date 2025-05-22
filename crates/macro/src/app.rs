pub mod routes;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_routing::RouteTree;

use crate::app::routes::{gen_module_declarations, gen_route_match_function};
use crate::helpers::get_project_dir;

pub fn ruxy_app(_config: impl Into<TokenStream>) -> proc_macro::TokenStream {
  let project_dir = get_project_dir();
  let routes_dir = project_dir.join("src/routes");

  let routes = RouteTree::new(&routes_dir);

  let module_declarations = gen_module_declarations(&routes);
  let route_match_function = gen_route_match_function(&routes);

  let main_ident = Ident::new("main", Span::call_site());
  let app_ident = Ident::new("App", Span::call_site());

  let errors = routes.get_compile_errors();
  let errors_rendered = render_errors(errors);
  
  let output = quote! {
    #module_declarations

    fn #main_ident() {
      struct #app_ident;

      impl ::ruxy::macro_internal::Runtime for App {
        #route_match_function
      };

      println!("Hello, world!");

      <#app_ident as ::ruxy::macro_internal::Runtime>::start();
    }
    
    #errors_rendered
  };

  output.into()
}

fn render_errors(errors: Vec<String>) -> TokenStream {
  if errors.is_empty() {
    return TokenStream::new();
  }

  let err_heading = format!(
    "Ruxy can not compile your application due to the following {count}error{plural}:",
    count = if errors.len() == 1 { "".to_owned() } else { format!(" {}", errors.len()) },
    plural = if errors.len() == 1 { "" } else { "s" }
  );

  let err_message = format!("{}\r\n{}", err_heading, errors.join("\r\n--------\r\n"));

  quote! { compile_error!(#err_message); }
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
