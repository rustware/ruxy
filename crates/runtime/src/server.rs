use std::future::Future;
use std::net::SocketAddr;

use hyper::service::service_fn;
use hyper::{Response, http};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn;
use tokio::net::TcpListener;

use crate::executor::AsyncExecutor;
use crate::response::body::ResponseBody;

pub type HyperRequest = hyper::Request<hyper::body::Incoming>;

/// This trait is the glue between the runtime and the `app!` macro.
/// Abstract methods are implemented by the `app!` macro, enabling the
/// expanded macro to call into the runtime and vice versa.
pub trait Server: Send + 'static {
  fn start() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
      // TODO: Make this configurable
      let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

      let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
          // TODO: Logging
          eprintln!("[ERR] failed to bind to '{}': {}", addr, e);
          std::process::exit(1);
        }
      };

      loop {
        match listener.accept().await {
          Ok((stream, _addr)) => {
            // TODO: Metrics

            let io = TokioIo::new(stream);

            tokio::spawn(async move {
              let builder = conn::auto::Builder::new(AsyncExecutor);

              if let Err(err) = builder.serve_connection(io, service_fn(Self::serve)).await {
                println!("[ERR] error serving connection: {:?}", err);
              }
            });
          }
          Err(e) => {
            // TODO: Logging, metrics
            eprintln!("[ERR] couldn't get client: {}", e);
            continue;
          }
        }
      }
    });
  }

  fn serve(req: HyperRequest) -> impl Future<Output = http::Result<Response<ResponseBody>>> + Send {
    async { Self::handler(req).await.response }
  }

  /// Implemented by the `app!` macro
  fn handler(req: HyperRequest) -> impl Future<Output = HandlerResult> + Send;

  /// Returns the remaining characters of the current segment from `path`,
  /// and the remaining characters of `path` after the segment end.
  /// 
  /// This function does NOT consume the trailing slash at the end of segment,
  /// and returns it as part of the remaining characters (_, <remaining>).
  /// 
  /// Example:
  /// `split_segment_end("remaining/rest/of/path")`
  /// returns `Some(("remaining", "/rest/of/path"))`
  #[inline]
  fn split_segment_end(path: &str, ) -> (&str, &str) {
     match path.find('/') {
      Some(i) => (&path[..i], &path[i..]),
      None => (path, ""),
    }
  }

  /// Strips the suffix of the current URL segment and returns a tuple containing
  /// the value before the suffix, and the rest of path.
  ///
  /// Example:
  /// `strip_segment_suffix("segment1-mysuffix/rest/of/path", "-mysuffix")`
  /// returns `Some(("segment1", "/rest/of/path"))`
  ///
  /// Returns `None` if the suffix was not found at the end of the current URL segment.
  #[inline]
  fn strip_segment_suffix<'a>(path: &'a str, suffix: &str) -> Option<(&'a str, &'a str)> {
    let (segment, rest) = Self::split_segment_end(path);
    segment.strip_suffix(suffix).map(|stripped| (stripped, rest))
  }
  
  /// Strips the leading slash of the provided `path`.
  #[inline]
  fn strip_prefix_slash(path: &str) -> &str {
    path.strip_prefix('/').unwrap_or(path)
  }
  
  /// Produces a response that redirects the user to the provided `path`.
  #[inline]
  fn redirect_to_path(request: &HyperRequest, path: &str) -> HandlerResult {
    todo!()
  }
  
  /// Produces a response that redirects the user to the provided `path` with added trailing slash.
  #[inline]
  fn redirect_to_added_slash(request: &HyperRequest, path: &str) -> HandlerResult {
    todo!()
  }
}

pub struct HandlerResult {
  pub response: http::Result<Response<ResponseBody>>,
}

#[cfg(test)]
mod tests {
  use super::*;

  struct TestServer;

  impl Server for TestServer {
    async fn handler(_req: HyperRequest) -> HandlerResult {
      HandlerResult { response: Ok(Response::new(ResponseBody::new())) }
    }
  }

  #[test]
  fn test_split_segment_end() {
    assert_eq!(TestServer::split_segment_end("remaining/rest/of/path"), ("remaining", "/rest/of/path"));
    assert_eq!(TestServer::split_segment_end("remaining/"), ("remaining", "/"));
    assert_eq!(TestServer::split_segment_end("remaining"), ("remaining", ""));
    assert_eq!(TestServer::split_segment_end("/rest"), ("", "/rest"));
    assert_eq!(TestServer::split_segment_end("/"), ("", "/"));
    assert_eq!(TestServer::split_segment_end(""), ("", ""));
  }

  #[test]
  fn test_strip_segment_suffix() {
    assert_eq!(TestServer::strip_segment_suffix("pre-suf/rest/of/path", "-suf"), Some(("pre", "/rest/of/path")));
    assert_eq!(TestServer::strip_segment_suffix("pre-suf", "-suf"), Some(("pre", "")));
    assert_eq!(TestServer::strip_segment_suffix("pre-suf/", "-suf"), Some(("pre", "/")));
    assert_eq!(TestServer::strip_segment_suffix("-suf", "-suf"), Some(("", "")));
    assert_eq!(TestServer::strip_segment_suffix("pre", ""), Some(("pre", "")));
    assert_eq!(TestServer::strip_segment_suffix("", ""), Some(("", "")));

    // No match
    assert_eq!(TestServer::strip_segment_suffix("random", "-suf"), None);
    assert_eq!(TestServer::strip_segment_suffix("random/", "-suf"), None);
    assert_eq!(TestServer::strip_segment_suffix("random/rest/of/path", "-suf"), None);

    // Matches & strips only the last occurence in the segment
    assert_eq!(TestServer::strip_segment_suffix("pre-suf-suf/rest", "-suf"), Some(("pre-suf", "/rest")));
  }
}
