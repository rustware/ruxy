use proc_macro2::TokenStream;
use quote::quote;

pub fn derive_props(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let ident = input.ident;
  
  let result = quote! {
    impl ::ruxy::Props for #ident { }
  };
  
  result.into()
}
