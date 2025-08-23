use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_main_function() -> TokenStream {
  quote! {
    fn main() -> impl std::process::Termination {
      // TODO: Call user-provided `main` function from `main.rs` (if exists)
    }
  }
}