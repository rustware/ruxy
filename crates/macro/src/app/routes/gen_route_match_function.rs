use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use std::str::FromStr;

use ::ruxy_routing::RouteTree;

pub fn gen_route_match_function(routes: &RouteTree) -> TokenStream {
  let ident = Ident::new("route", Span::mixed_site());
  let hyper_req_ident = Ident::new("hyper_req", Span::mixed_site());

  quote! {
    fn #ident(#hyper_req_ident: ::ruxy::macro_internal::HyperRequest) {

    }
  }
}
