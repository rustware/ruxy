use proc_macro2::TokenStream;

mod parse;

/// Parsed input of the `app!` macro.
pub struct AppMacroInput {}

impl TryFrom<TokenStream> for AppMacroInput {
  type Error = TokenStream;

  fn try_from(value: TokenStream) -> Result<Self, Self::Error> {
    parse::parse_app_macro_input(value)
  }
}
