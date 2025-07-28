use proc_macro2::TokenStream;

use super::BuildMacroInput;

pub fn parse_app_macro_input(_input: TokenStream) -> Result<BuildMacroInput, TokenStream> {
  Ok(BuildMacroInput {})
}
