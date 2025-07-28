mod router;
mod trailing_slash;

use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::route_tree::RouteTree;

use crate::app::input::AppMacroInput;

pub fn gen_handler_function(config: &AppMacroInput, routes: &RouteTree) -> TokenStream {
  let router = router::generate(config, routes);
  let router = trailing_slash::wrap_router(router);

  quote! {
    async fn handler(request: internal::HyperRequest) -> internal::HandlerResult {
      let path = request.uri().path();
      #router
    }
  }
}
