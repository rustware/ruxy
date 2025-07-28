mod input;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_routing::route_tree::RouteTree;
use ::ruxy_util::fs::get_project_dir;

use crate::build::input::BuildMacroInput;
use crate::helpers::render_routes_watch;

pub fn ruxy_build(input: TokenStream) -> Result<TokenStream, TokenStream> {
  let _input: BuildMacroInput = input.try_into()?;

  let output = quote! {
    fn main() {
      
    }
  };

  Ok(output)
}
