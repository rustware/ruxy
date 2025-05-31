use proc_macro2::TokenStream;
use quote::quote;

pub fn render_errors(errors: Vec<String>) -> TokenStream {
  if errors.is_empty() {
    return TokenStream::new();
  }

  let err_heading = format!(
    "Ruxy can not compile your application due to the following {count}error{plural}:\r\n–––\r\n",
    count = if errors.len() == 1 { "".to_owned() } else { format!("{} ", errors.len()) },
    plural = if errors.len() == 1 { "" } else { "s" }
  );

  let err_message = format!("{}\r\n{}", err_heading, errors.join("\r\n–––\r\n"));

  quote! { compile_error!(#err_message); }
}
