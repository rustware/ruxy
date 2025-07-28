pub mod page;
pub mod request;
pub mod response;
pub mod server;

mod routing;
mod redirect;
mod not_found;
mod acceptor;

pub use redirect::redirect;

#[doc(hidden)]
pub mod __ruxy_macro_internal {
  pub use bytes::Bytes;
  pub use hyper::body::Frame;

  pub use ::ruxy_config::register_app_config;

  pub use super::server::HandlerResult;
  pub use super::server::HyperRequest;
  pub use super::server::Server;

  pub use super::response::body::ResponseBody;
}
