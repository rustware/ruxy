use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::route_tree::RouteTree;

use crate::helpers::get_route_file;

pub fn ruxy_page(input: impl Into<TokenStream>, _args: impl Into<TokenStream>) -> proc_macro::TokenStream {
  let route_file = get_route_file();

  if !route_file.file_type.is_page() {
    panic!("This macro can only be used in a page file – `page.rs`, `error_page.rs`, or `not_found_page.rs`.");
  }
  
  // TODO: Extract path parameters from `route_fils.segment_id` by parsing the segment recursively
  // TODO: Wrap the decorated function with a wrapper function that can extract stuff from the request

  let input = input.into();

  let output = quote! {
    #input
  };

  output.into()
}
