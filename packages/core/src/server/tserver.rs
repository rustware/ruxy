use std::future::Future;
use std::net::SocketAddr;
use std::process::Termination;
use std::str::FromStr;

use hyper::http::HeaderValue;
use hyper::service::service_fn;
use hyper::{Response, StatusCode, http};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn;
use tokio::net::{TcpListener, TcpStream};

use crate::config::get_app_config;

use crate::server::response::body::ResponseBody;

pub type HyperRequest = hyper::Request<hyper::body::Incoming>;

/// This trait is the glue between the runtime and the `app!` macro.
/// Abstract methods are implemented by the `app!` macro, enabling the
/// expanded macro to call into the runtime and vice versa.
#[allow(async_fn_in_trait)]
pub trait Server: Send + 'static {
  // This default is pretty arbitrary, we'll adjust it in time when we have more
  // data about real production usage and the performance of the server. Anyway,
  // this can be easily overridden by the user.
  const REQUEST_QUEUE_SIZE: usize = 10;

  fn start() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    rt.block_on(async {
      let app_config = get_app_config();
      
      for (index, address) in app_config.addresses.iter().enumerate() {
        // Addresses are validated in the parsing step, so we can just unwrap here.
        let address = SocketAddr::from_str(address).unwrap();

        // We don't want to start listening in here, we want to spawn a new task for that:
        // https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#non-worker-future
        let handle = tokio::task::spawn(Self::listen(address));

        if index == app_config.addresses.len() - 1 {
          // TODO: Logs
          handle.await.expect("error starting the server");
        }
      }
    });
  }

  fn listen(address: SocketAddr) -> impl Future<Output = ()> + Send {
    async move {
      let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(e) => {
          // TODO: Logs
          eprintln!("[ERR] failed to bind to '{address}': {e}");
          std::process::exit(1);
        }
      };

      loop {
        match listener.accept().await {
          Ok((stream, _addr)) => {
            // TODO: Metrics
            Self::process_tcp_stream(stream);
          }
          Err(e) => {
            // TODO: Logs, metrics
            eprintln!("[ERR] failed to establish connection: {e}");
            continue;
          }
        }
      }
    }
  }

  fn process_tcp_stream(stream: TcpStream) {
    let io = TokioIo::new(stream);

    tokio::task::spawn(async move {
      let builder = conn::auto::Builder::new(crate::runtime::AsyncExecutor);

      if let Err(err) = builder.serve_connection(io, service_fn(Self::serve)).await {
        // TODO: Logs
        println!("[ERR] error serving connection: {err:?}");
      }
    });
  }

  fn serve(req: HyperRequest) -> impl Future<Output = http::Result<Response<ResponseBody>>> + Send {
    async { Self::handler(req).await.response }
  }

  /// Implemented by the `app!` macro
  fn handler(req: HyperRequest) -> impl Future<Output = HandlerResult> + Send;

  /// Implemented by the `app!` macro
  fn main() -> impl Termination;

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

    // We can just unwrap here, we know the path is valid UTF-8.
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
