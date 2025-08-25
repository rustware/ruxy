mod global_404;
mod responder;
mod matcher;
mod trailing_slash;

use proc_macro2::TokenStream;
use quote::quote;

use crate::build::app::context::GenContext;

pub fn gen_handler_functions(ctx: &GenContext) -> TokenStream {
  let matcher = matcher::gen_matcher(ctx);
  let matcher = trailing_slash::wrap_matcher(matcher);
  
  let global_404 = global_404::gen_global_404();

  
  
  quote! {
    async fn handler(request: internal::HyperRequest) -> internal::HandlerResult {
      let path = request.uri().path();
      #matcher
      #global_404
    }
    
    
  }
}
