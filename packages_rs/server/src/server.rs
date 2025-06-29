use std::future::Future;
use std::net::SocketAddr;

use hyper::http::HeaderValue;
use hyper::service::service_fn;
use hyper::{Response, StatusCode, http};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn;
use tokio::net::{TcpListener, TcpStream};

use ::ruxy_runtime::threads;

use crate::response::body::ResponseBody;

pub type HyperRequest = hyper::Request<hyper::body::Incoming>;

/// This trait is the glue between the runtime and the `app!` macro.
/// Abstract methods are implemented by the `app!` macro, enabling the
/// expanded macro to call into the runtime and vice versa.
#[allow(async_fn_in_trait)]
pub trait Server: Send + 'static {
  // This default is pretty arbitrary, we'll adjust it in time when we have more
  // data about real production usage and the performance of the server. Anyway,
  // it's just a starting point that can be adjusted by passing relevant config.
  const REQUEST_QUEUE_SIZE: usize = 5;

  fn start() {
    let thread_count = threads::get_thread_count();

    let main = tokio::runtime::Builder::new_current_thread().enable_io().build().unwrap();

    let (tx, rx) = flume::bounded::<TcpStream>(Self::REQUEST_QUEUE_SIZE);

    for _ in 0..thread_count - 1 {
      let rx = rx.clone();

      std::thread::spawn(move || {
        let worker = tokio::runtime::Builder::new_current_thread().build().unwrap();

        worker.block_on(async {
          let local = tokio::task::LocalSet::new();

          let root_task = local.run_until(async move {
            while let Ok(stream) = rx.recv_async().await {
              dbg!("[INFO] processing on worker thread");
              Self::process_tcp_stream(stream);
            }
          });
          
          root_task.await;
        });
      });
    }

    main.block_on(async move {
      let local = tokio::task::LocalSet::new();

      let root_task = local.run_until(async move {
        // TODO: Make this configurable
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

        let listener = match TcpListener::bind(addr).await {
          Ok(listener) => listener,
          Err(e) => {
            // TODO: Logs
            eprintln!("[ERR] failed to bind to '{addr}': {e}");
            std::process::exit(1);
          }
        };

        // This condition is intentionally outside the loops, as
        // those are hot loops, so we avoid one extra branch inside.
        if thread_count == 1 {
          // Single-threaded hot-loop
          loop {
            let stream = Self::listen(&listener).await;
            dbg!("[INFO] processing on main thread");
            Self::process_tcp_stream(stream);
          }
        } else {
          // Multi-threaded hot-loop
          loop {
            let stream = Self::listen(&listener).await;

            dbg!("[INFO] sending socket to worker thread");
            if tx.send_async(stream).await.is_err() {
              // All receivers dropped, we're shutting down.
              break;
            }
          }
        }
      });
      
      root_task.await;
    });
  }
  
  async fn listen(listener: &TcpListener) -> TcpStream {
    // Only loops until there's a valid (non-error) connection.
    loop {
      match listener.accept().await {
        Ok((stream, _addr)) => {
          // TODO: Metrics
          return stream;
        }
        Err(e) => {
          // TODO: Logs, metrics
          eprintln!("[ERR] couldn't get client: {e}");
        }
      }
    }
  }

  fn process_tcp_stream(stream: TcpStream) {
    tokio::task::spawn_local(async move {
      let io = TokioIo::new(stream);

      let builder = conn::auto::Builder::new(ruxy_runtime::AsyncExecutor);

      if let Err(err) = builder.serve_connection(io, service_fn(Self::serve)).await {
        // TODO: Logs
        println!("[ERR] error serving connection: {err:?}");
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
