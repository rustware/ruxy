pub mod page;
pub mod request;
pub mod response;
pub mod tserver;

mod routing;
mod redirect;
mod not_found;
mod acceptor;

pub use redirect::redirect;

// Re-exports for usage by "ruxy" crate
pub mod re {
  pub use bytes::Bytes;
  pub use hyper::body::Frame;

  pub use super::tserver::HandlerResult;
  pub use super::tserver::HyperRequest;
  pub use super::tserver::Server;

  pub use super::response::body::ResponseBody;
}
