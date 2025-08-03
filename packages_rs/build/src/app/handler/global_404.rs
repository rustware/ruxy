use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_global_404() -> TokenStream {
  quote! {
    let mut response = hyper::Response::builder();

    response = response.status(404);
    response = response.header("Content-Type", "text/plain");

    let mut body = internal::ResponseBody::new();

    body.push(internal::Bytes::from("Not Found"));

    internal::HandlerResult {
      response: response.body(body)
    }
  }
}
