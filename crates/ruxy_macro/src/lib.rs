mod app;

use proc_macro::TokenStream;

#[proc_macro]
pub fn app(input: TokenStream) -> TokenStream {
  app::ruxy_app(input)
}
