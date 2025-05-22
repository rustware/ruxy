use std::net::SocketAddr;

use http_body_util::{BodyStream, StreamBody};
use hyper::service::service_fn;
use hyper::{Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn;
use tokio::net::TcpListener;

use crate::executor::AsyncExecutor;

pub type HyperRequest = hyper::Request<hyper::body::Incoming>;

/// This trait is a "bridge" between the runtime and the `app!` macro.
/// Abstract methods are implemented by the `app!` macro, enabling the
/// expanded macro to call into the runtime and vice versa.
///
/// Since this trait is used by the `app!` macro, the user code has to
/// be able to import it, but it should never be called by the user
/// directly, thus `#[doc(hidden)]` is applied.
#[doc(hidden)]
pub trait Runtime: Sized + Send + 'static {
  fn start() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
      // TODO: Make this configurable:
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
            let builder = conn::auto::Builder::new(AsyncExecutor);

            if let Err(err) = builder.serve_connection(io, service_fn(Self::serve)).await {
              println!("[ERR] error serving connection: {:?}", err);
            }
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

  fn serve(_req: HyperRequest) -> impl Future<Output = ServeResult> + Send + 'static {
    async {
      // 1. Routing + Gathering Server Values goes here
      // TODO: Routing + Gathering Server Values

      // 2. Start rendering and send the ready chunks to the client
      // TODO: Rendering + Streaming Response
      let stream = StreamBody::new(BodyStream::new("Hello, world!".to_owned()));
      Response::builder().body(stream)
    }
  }

  fn route(req: HyperRequest) {}
}

type ServeResult = hyper::http::Result<Response<StreamBody<BodyStream<String>>>>;
