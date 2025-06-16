use proc_macro2::TokenStream;
use quote::quote;

pub fn render_errors(errors: Vec<String>) -> TokenStream {
  if errors.is_empty() {
    return TokenStream::new();
  }

  let errors_head = format!(
    "Ruxy cannot compile your application due to the following {count}error{plural}:",
    count = if errors.len() == 1 { "".to_owned() } else { format!("{} ", errors.len()) },
    plural = if errors.len() == 1 { "" } else { "s" }
  );

  let errors = errors.iter().enumerate().map(|(i, e)| {
    let mut error = format!("\n––––– Error #{} –––––\n", i + 1);
    error.push_str(e);
    error
  });
  
  let err_message = format!("{}{}", errors_head, errors.collect::<Vec<_>>().join(""));

  quote! { compile_error!(#err_message); }
}
