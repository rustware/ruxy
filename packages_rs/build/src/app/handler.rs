mod router;
mod trailing_slash;
mod responder;
mod global_404;
mod generator;

use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::route_tree::RouteTree;

pub fn gen_handler_function(routes: &RouteTree) -> TokenStream {
  let router = router::gen_router(routes);
  let router = trailing_slash::wrap_router(router);

  // Generate a global 404 handler
  let global_404 = global_404::gen_global_404();

  quote! {
    async fn handler(request: internal::HyperRequest) -> internal::HandlerResult {
      let path = request.uri().path();
      #router
      #global_404
    }
  }
}
