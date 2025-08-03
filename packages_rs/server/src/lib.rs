pub mod page;
pub mod request;
pub mod response;
pub mod server;

mod routing;
mod redirect;
mod not_found;
mod acceptor;

pub use redirect::redirect;

// Re-exports for usage by "ruxy" crate
pub mod re {
  pub use bytes::Bytes;
  pub use hyper::body::Frame;

  pub use super::server::HandlerResult;
  pub use super::server::HyperRequest;
  pub use super::server::Server;

  pub use super::response::body::ResponseBody;
}
