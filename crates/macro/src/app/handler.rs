mod trailing_slash;
mod router;

use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::RouteTree;

use crate::app::config::AppConfig;

pub fn gen_handler_function(config: &AppConfig, routes: &RouteTree) -> TokenStream {
  let router = router::generate(config, routes);
  let router = trailing_slash::wrap_router(config, router);

  quote! {
    async fn handler(request: internal::HyperRequest) -> internal::HandlerResult {
      let path = request.uri().path();
      #router
    }
  }
}
