use crate::helpers::handle_inquire_error;

use super::{ClientLanguage, CmdInitOptions};

pub fn get_client_language(options: &CmdInitOptions) -> ClientLanguage {
  match options.enable_typescript {
    Some(true) => ClientLanguage::TS,
    Some(false) => ClientLanguage::JS,
    None => prompt_client_language(),
  }
}

fn prompt_client_language() -> ClientLanguage {
  let input = inquire::Confirm::new("Do you want to enable TypeScript support in your client code?").with_default(true);

  match input.prompt() {
    Ok(true) => ClientLanguage::TS,
    Ok(false) => ClientLanguage::JS,
    Err(err) => handle_inquire_error(err),
  }
}
