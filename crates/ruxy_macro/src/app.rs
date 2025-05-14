pub mod routes;

use quote::quote;
use proc_macro::TokenStream;

use crate::app::routes::Routes;

pub fn ruxy_app(_config: TokenStream) -> TokenStream {
  let routes = Routes::create();

  let modules = routes.get_module_declarations();
  let routing = routes.get_routing();
  
  let output = quote! {
    #modules
    
    fn main() {
      println!("test");
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
