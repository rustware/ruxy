use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use crate::helpers::{get_params_for_segment_id, get_route_file};

pub fn page_loader(_args: proc_macro::TokenStream, func: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let route_file = get_route_file();

  let mut errors = vec![];

  if !route_file.file_type.is_page() {
    errors.push("this macro can only be used in a page file – `page.rs`, `error_page.rs`, or `not_found_page.rs`.");
  }

  let path_params = get_params_for_segment_id(&route_file.segment_id);
  let path_params_len = path_params.len();
  let path_params_type = quote! { [&str; #path_params_len] };
  let path_params_ident = Ident::new("__ruxy_path_params", Span::mixed_site());

  let ruxy_fn_ident = Ident::new("__ruxy_page", Span::mixed_site());

  let return_type =
    quote! { ()/*::std::result::Result<::ruxy::__ruxy_macro_internal::Response, ::std::error::Error>*/ };

  let input = parse_macro_input!(func as ItemFn);
  let is_async = input.sig.asyncness.is_some();

  let user_fn_ident = &input.sig.ident;

  let user_fn_call = quote! { self::#user_fn_ident() };

  // Add the `.await` postfix if the user function is async.
  let user_fn_call = if is_async {
    quote! { #user_fn_call.await }
  } else {
    user_fn_call
  };

  let errors = match errors.is_empty() {
    true => TokenStream::new(),
    false => {
      let errors = errors.join("\n\n");
      quote! { compile_error!(#errors); }
    }
  };

  let output = quote! {
    #input

    #[doc(hidden)]
    pub(in crate::app) async fn #ruxy_fn_ident(#path_params_ident: #path_params_type) -> #return_type {
      let user_fn_result = #user_fn_call;
    }

    #errors
  };

  output.into()
}
