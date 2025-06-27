use std::future::Future;
use std::net::SocketAddr;

use hyper::http::HeaderValue;
use hyper::service::service_fn;
use hyper::{Response, StatusCode, http};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn;
use tokio::net::TcpListener;

use crate::response::body::ResponseBody;

pub type HyperRequest = hyper::Request<hyper::body::Incoming>;

/// This trait is the glue between the runtime and the `app!` macro.
/// Abstract methods are implemented by the `app!` macro, enabling the
/// expanded macro to call into the runtime and vice versa.
pub trait Server: Send + 'static {
  fn start() {
    // TODO: Multiple threads, early load-balancing
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    
    rt.block_on(async {
      // TODO: Make this configurable
      let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

      let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
          // TODO: Logging
          eprintln!("[ERR] failed to bind to '{addr}': {e}");
          std::process::exit(1);
        }
      };

      loop {
        match listener.accept().await {
          Ok((stream, _addr)) => {
            // TODO: Metrics

            let io = TokioIo::new(stream);

            // Do we need to spawn a task here? `AsyncExecutor` spawns a task for us.
            tokio::task::spawn_local(async move {
              let builder = conn::auto::Builder::new(ruxy_runtime::hyper::AsyncExecutor);

              if let Err(err) = builder.serve_connection(io, service_fn(Self::serve)).await {
                println!("[ERR] error serving connection: {err:?}");
              }
            });
          }
          Err(e) => {
            // TODO: Logging, metrics
            eprintln!("[ERR] couldn't get client: {e}");
            continue;
          }
        }
      }
    });
  }

  fn serve(req: HyperRequest) -> impl Future<Output = http::Result<Response<ResponseBody>>> {
    async { Self::handler(req).await.response }
  }

  /// Implemented by the `app!` macro
  fn handler(req: HyperRequest) -> impl Future<Output = HandlerResult>;

  /// Produces a response that redirects the user to the provided `path`.
  #[inline]
  fn redirect_to_path(path: &str) -> HandlerResult {
    let path = if path.is_empty() { "/" } else { path };
    
    HandlerResult {
      response: http::Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header(http::header::LOCATION, path)
        .body(ResponseBody::new()),
    }
  }

  /// Produces a response that redirects the user to the provided `path` with added trailing slash.
  #[inline]
  fn redirect_to_added_slash(path: &str) -> HandlerResult {
    let mut location = Vec::with_capacity(path.len() + 1);
    location.extend_from_slice(path.as_bytes());
    location.extend_from_slice(b"/");

    // We can just unwrap here because we know the path is valid UTF-8.
    let location = HeaderValue::from_bytes(location.as_slice()).unwrap();

    HandlerResult {
      response: http::Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header(http::header::LOCATION, location)
        .body(ResponseBody::new()),
    }
  }
}

pub struct HandlerResult {
  pub response: http::Result<Response<ResponseBody>>,
}
