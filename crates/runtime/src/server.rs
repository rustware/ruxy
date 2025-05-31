use std::future::Future;
use std::net::SocketAddr;

use crate::executor::AsyncExecutor;
use crate::response::body::ResponseBody;
use hyper::service::service_fn;
use hyper::{Response, http};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn;
use ruxy_util::decode_hex_pair;
use tokio::net::TcpListener;

pub type HyperRequest = hyper::Request<hyper::body::Incoming>;

/// This trait is the glue between the runtime and the `app!` macro.
/// Abstract methods are implemented by the `app!` macro, enabling the
/// expanded macro to call into the runtime and vice versa.
pub trait Server: Send + 'static {
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

  /// Strips a literal prefix, like `&str.strip_prefix`, while decoding all
  /// URL-encoded bytes in the `url`. This does not allocate, it simply compares
  /// the relevant bytes (decoded if needed) and moves the slice pointer.
  fn strip_prefix_decode<'a>(url: &'a str, prefix: &str) -> Option<&'a str> {
    let prefix_bytes = prefix.as_bytes().iter();
    let mut url_bytes = url.as_bytes().iter();

    let mut skip_url_bytes_count = 0;
    
    for prefix_byte in prefix_bytes {
      let url_byte = url_bytes.next()?;

      // Handle literal byte

      if url_byte != &b'%' {
        if prefix_byte != url_byte {
          return None;
        }
        
        continue;
      }

      // Handle encoded sequence

      let (hex1, hex2) = (url_bytes.next()?, url_bytes.next()?);
      let byte = decode_hex_pair(*hex1, *hex2)?;

      if prefix_byte != &byte {
        return None;
      }
      
      // Skip the two bytes we just "consumed" from the URL
      skip_url_bytes_count += 2;
    }

    Some(&url[prefix.len() + skip_url_bytes_count..])
  }
  
  fn decode_dyn_segment_value(value: &str) -> String {
    match urlencoding::decode(value) {
      Ok(decoded) => decoded.to_string(),
      _ => value.to_string(),
    }
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
  fn test_strip_prefix_resolved() {
    // Test empty prefix
    assert_eq!(TestServer::strip_prefix_decode("/path", ""), Some("/path"));

    // Test simple prefix
    assert_eq!(TestServer::strip_prefix_decode("/path/to/resource", "/path"), Some("/to/resource"));

    // Test no match
    assert_eq!(TestServer::strip_prefix_decode("/path", "/other"), None);

    // Test exact match
    assert_eq!(TestServer::strip_prefix_decode("/path", "/path"), Some(""));

    // Test incomplete match (-> no match)
    assert_eq!(TestServer::strip_prefix_decode("/prefix-is", "/prefix-is-longer"), None);

    // Test with percent encoding
    assert_eq!(TestServer::strip_prefix_decode("/path%20with%20spaces", "/path "), Some("with%20spaces"));

    // Test with percent encoding in both (should be matched literally)
    assert_eq!(TestServer::strip_prefix_decode("/path%2520rest", "/path%20r"), Some("est"));

    // Test with percent encoding in both (no match)
    assert_eq!(TestServer::strip_prefix_decode("/path rest", "/path%20"), None);

    // Test with percent encoding with single special literal in prefix
    assert_eq!(TestServer::strip_prefix_decode("%25/", "%"), Some("/"));

    // Test with percent encoding at the boundary
    assert_eq!(TestServer::strip_prefix_decode("/path%20/resource", "/path"), Some("%20/resource"));

    // Test with invalid percent encoding
    assert_eq!(TestServer::strip_prefix_decode("/path%2rest", "/path"), Some("%2rest"));
    assert_eq!(TestServer::strip_prefix_decode("/path%2rest", "/"), Some("path%2rest"));
    assert_eq!(TestServer::strip_prefix_decode("/path%2Grest", "/path%2G"), None);

    // Test with incomplete Unicode code point
    assert_eq!(TestServer::strip_prefix_decode("/path%D1rest", "/path%"), None);

    // Test with multi-byte characters
    assert_eq!(TestServer::strip_prefix_decode("/a-%D1%BF/", "/a-ѿ"), Some("/"));
    assert_eq!(TestServer::strip_prefix_decode("%D1%BF%25", "ѿ%"), Some(""));
  }
}
