use proc_macro2::TokenStream;

use super::AppMacroInput;

pub fn parse_app_macro_input(_input: TokenStream) -> Result<AppMacroInput, TokenStream> {
  // Reserved for future use
  Ok(AppMacroInput {})
}
