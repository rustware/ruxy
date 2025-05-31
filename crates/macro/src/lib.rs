mod app;
mod helpers;
mod page;

use proc_macro::TokenStream;

/// Ruxy's `app!` macro powers your application.
///
/// In does multiple fundamental things to make the application work, such as:
/// 1. Discovering routes by parsing the `routes/` directory,
/// 1. building allocation-free routing layer based on the discovered routes,
/// 1. building allocation-free rendering layer, connecting RIR with routes and handlers,
/// 1. providing the application entry point,
/// 1. glueing everything together.
///
/// A certain class of optional configuration affecting the runtime of your application
/// can be put inside this macro, as documented at https://ruxy.dev/docs/files/app_rs.
#[proc_macro]
pub fn app(input: TokenStream) -> TokenStream {
  app::ruxy_app(input)
}

/// Ruxy's `page` attribute is how you let Ruxy know about your page handler function,
/// and how you tweak the exact behavior of how your page processes incoming requests.
/// 
/// Ruxy will help you inject arguments into your function using special attributes,
/// ensuring you only consume exactly what you need. This way we can prevent expensive
/// deserialization of those parts of the request that you're not interested in.
/// 
/// Example:
/// ```
/// // #[ruxy::page]
/// // pub fn page(#[ctx] ctx, #[headers] headers, #[header("Header-Name") header_name]) {
/// //   use context or headers
/// // }
/// ```
#[proc_macro_attribute]
pub fn page(input: TokenStream, args: TokenStream) -> TokenStream {
  page::ruxy_page(input, args)
}
