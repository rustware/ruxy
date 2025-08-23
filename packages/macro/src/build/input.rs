use proc_macro2::TokenStream;

mod parse;

/// Parsed input of the `build!` macro.
pub struct BuildMacroInput {}

impl TryFrom<TokenStream> for BuildMacroInput {
  type Error = TokenStream;

  fn try_from(value: TokenStream) -> Result<Self, Self::Error> {
    parse::parse_app_macro_input(value)
  }
}
