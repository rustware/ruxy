mod app;
mod helpers;

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
