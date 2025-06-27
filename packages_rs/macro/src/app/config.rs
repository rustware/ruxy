use proc_macro2::TokenStream;

pub struct MacroConfig {}

impl Default for MacroConfig {
  fn default() -> Self {
    Self {}
  }
}

pub fn parse_macro_config(_input: TokenStream) -> MacroConfig {
  let config: MacroConfig = Default::default();

  // TODO: Parse macro input and update `config`

  config
}
