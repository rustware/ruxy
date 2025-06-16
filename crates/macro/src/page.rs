use proc_macro2::{TokenStream};
use quote::quote;

use ::ruxy_routing::route_tree::RouteTree;

use crate::helpers::get_project_dir;

pub fn ruxy_page(input: impl Into<TokenStream>, _args: impl Into<TokenStream>) -> proc_macro::TokenStream {
  let project_dir = get_project_dir();
  let routes_dir = project_dir.join("src/routes");
  let cache_dir = project_dir.join(".ruxy");

  let routes = RouteTree::new(&routes_dir);

  // TODO
  
  let input = input.into();
  
  let output = quote! {
    #input
  };

  output.into()
}
