pub mod page;
pub mod request;
pub mod response;
pub mod server;

mod executor;
mod routing;

#[doc(hidden)]
pub mod macro_internal {
  pub use bytes::Bytes;
  pub use hyper::body::Frame;

  pub use super::server::HandlerResult;
  pub use super::server::HyperRequest;
  pub use super::server::Server;

  pub use super::response::body::ResponseBody;
}

// 1. Static matches (strip_prefix)
// 2. Dynamic matches
// 3. Closest 404 handler
// 4. Default 404 handler

fn t_route(path: &str, r: server::HyperRequest) {
  if let Some(path) = path.strip_prefix("/hel") {
    // 1. Static matches
    if let Some(path) = path.strip_prefix("lo") {}

    if let Some(path) = path.strip_prefix("sinki") {}

    // 2. No dynamic possibilities here

    // 3. 404 cascade here
    return;
  }

  if path.starts_with("/hel") {
    // SAFETY: we just checked the presence of the prefix
    let path = unsafe { path.strip_prefix("/hel").unwrap_unchecked() };

    if path.starts_with("/lo") {
      if path.is_empty() {
        // /hello route matched
        // only include this condition when there's a route for /hello
        return;
      }

      // layout matched

      // 1. static matches for leafs:
      if path == "something" {
        // ...
      }

      // 2. static matches for non-leafs (radix trie):
      if path.starts_with("/som") {
        // ...
      }

      // 3. dynamic segments

      // not found
      return;
    }

    if path.starts_with("/sinki") {
      // route matched
    }
  }

  // there is only 1 route inside /api
  if path == "/api/test" {
    // route matched
  }
}
