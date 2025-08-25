pub mod page;
pub mod request;
pub mod response;
pub mod tserver;

mod routing;
mod redirect;
mod not_found;
mod acceptor;

pub use redirect::redirect;
