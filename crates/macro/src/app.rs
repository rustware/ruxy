pub mod routes;

use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::RouteTree;

use crate::app::routes::{gen_module_declarations, gen_routing_function};
use crate::helpers::get_project_dir;

pub fn ruxy_app(_config: impl Into<TokenStream>) -> proc_macro::TokenStream {
  let project_dir = get_project_dir();
  let routes_dir = project_dir.join("src/routes");
  
  let routes = RouteTree::new(&routes_dir);

  let module_declarations = gen_module_declarations(&routes);
  let routing_function = gen_routing_function(&routes);
  
  let output = quote! {
    #module_declarations

    fn main() {
      #routing_function
      
      struct App;
      
      impl ::ruxy::Runtime for App {
        
      };
      
      println!("Hello, world!");
      
      ::ruxy::Runtime::start(App)
    }
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
