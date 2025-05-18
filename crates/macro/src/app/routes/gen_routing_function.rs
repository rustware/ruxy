use proc_macro2::TokenStream;
use quote::quote;

use std::str::FromStr;

use ::ruxy_routing::{RouteTree, RouteSegment, RouteSegmentFileModule, RequestHandler};

pub fn gen_routing_function(routes: &RouteTree) -> TokenStream {
  quote! {
    async fn route(req: ::ruxy::Request) {
      // TODO: Bunch of match statements here composing a radix trie
    }
  }
}
