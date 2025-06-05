use std::future::Future;
use std::net::SocketAddr;

use hyper::service::service_fn;
use hyper::{Response, http};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn;
use tokio::net::TcpListener;

use ::ruxy_util::decode_hex_pair;

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

  /// Strips a literal prefix, like `&str.strip_prefix`, while decoding all
  /// URL-encoded bytes in the `url`. This does not allocate, it simply compares
  /// the relevant bytes (decoded if needed) and moves the slice pointer.
  fn strip_prefix<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let prefix_bytes = prefix.as_bytes().iter();
    let path_bytes = path.as_bytes();
    
    let mut path_bytes_iter = path_bytes.iter().enumerate();

    let mut skip_path_bytes_count = 0;

    for prefix_byte in prefix_bytes {
      let (path_byte_index, path_byte) = path_bytes_iter.next()?;

      let path_byte = if *path_byte == b'%' {
        let (hex1, hex2) = (path_bytes.get(path_byte_index + 1)?, path_bytes.get(path_byte_index + 2)?);

        if *hex1 == b'2' && (*hex2 == b'F' || *hex2 == b'f') {
          // We don't want to decode slashes, those are the only encoded characters
          // in UrlMatcher sequences, so we want to compare the encoded version here.
          *path_byte
        } else {
          skip_path_bytes_count += 2;
          let ((_, h1), (_, h2)) = (path_bytes_iter.next()?, path_bytes_iter.next()?);
          decode_hex_pair(*h1, *h2)?
        }
      } else {
        *path_byte
      };

      // Handle literal byte

      if *prefix_byte != path_byte {
        return None;
      }
    }

    Some(&path[prefix.len() + skip_path_bytes_count..])
  }

  fn decode_dyn_segment_value(value: &str) -> String {
    match urlencoding::decode(value) {
      Ok(decoded) => decoded.to_string(),
      _ => value.to_string(),
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
  ///
  /// This method walks through all bytes exactly once (single-pass), reducing CPU cycles.
  ///
  /// IMPORTANT: The suffix MUST be a non-empty UTF-8 string, otherwise this method panics.
  /// This is for performance reasons. Ruxy only calls this method inside the `app` macro
  /// with a macro-generated static values, so this invariant should always be upheld.
  fn strip_segment_suffix<'a>(path: &'a str, suffix: &str) -> Option<(&'a str, &'a str)> {
    let suffix_bytes = suffix.as_bytes();
    let path_bytes = path.as_bytes();

    let mut path_bytes_iter = path_bytes.iter().enumerate();

    let mut next_suffix_byte_index = 0;
    let mut matched_suffix_start_path_index = 0;
    let mut skip_decoded_bytes_count = 0;

    while let Some((path_byte_index, path_byte)) = path_bytes_iter.next() {
      if *path_byte == b'/' {
        break;
      }

      let byte = if *path_byte == b'%' {
        let (hex1, hex2) = (path_bytes.get(path_byte_index + 1)?, path_bytes.get(path_byte_index + 2)?);

        if *hex1 == b'2' && (*hex2 == b'F' || *hex2 == b'f') {
          // We don't want to decode slashes, those are the only encoded characters
          // in UrlMatcher sequences, so we want to compare the encoded version here.
          *path_byte
        } else {
          skip_decoded_bytes_count += 2;
          let ((_, h1), (_, h2)) = (path_bytes_iter.next()?, path_bytes_iter.next()?);
          decode_hex_pair(*h1, *h2)?
        }
      } else {
        *path_byte
      };

      let suffix_byte = suffix_bytes.get(next_suffix_byte_index).unwrap_or_else(|| {
        skip_decoded_bytes_count = 0;
        next_suffix_byte_index = 0;
        &suffix_bytes[0]
      });

      if byte == *suffix_byte {
        if next_suffix_byte_index == 0 {
          matched_suffix_start_path_index = path_byte_index;
        }

        next_suffix_byte_index += 1;
      } else {
        skip_decoded_bytes_count = 0;
        next_suffix_byte_index = 0;
      }
    }

    // End of segment

    if next_suffix_byte_index == suffix_bytes.len() {
      return Some((
        &path[..matched_suffix_start_path_index],
        &path[matched_suffix_start_path_index + next_suffix_byte_index + skip_decoded_bytes_count..],
      ));
    }

    None
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
  fn test_strip_prefix_decode() {
    // Test empty prefix
    assert_eq!(TestServer::strip_prefix("/path", ""), Some("/path"));

    // Test simple prefix
    assert_eq!(TestServer::strip_prefix("/path/to/resource", "/path"), Some("/to/resource"));

    // Test no match
    assert_eq!(TestServer::strip_prefix("/path", "/other"), None);

    // Test exact match
    assert_eq!(TestServer::strip_prefix("/path", "/path"), Some(""));

    // Test incomplete match => no match
    assert_eq!(TestServer::strip_prefix("/prefix-is", "/prefix-is-longer"), None);

    // Test with percent encoding
    assert_eq!(TestServer::strip_prefix("/path%20with%20spaces", "/path "), Some("with%20spaces"));

    // Test with percent encoding in both (should be matched literally)
    assert_eq!(TestServer::strip_prefix("/path%2520rest", "/path%20r"), Some("est"));

    // Test with percent encoding in both (no match)
    assert_eq!(TestServer::strip_prefix("/path rest", "/path%20"), None);

    // Test with percent encoding with single special literal in prefix
    assert_eq!(TestServer::strip_prefix("%25/", "%"), Some("/"));

    // Test with percent encoding at the boundary
    assert_eq!(TestServer::strip_prefix("/path%20/resource", "/path"), Some("%20/resource"));

    // Test with percent encoded slash
    assert_eq!(TestServer::strip_prefix("escaped%2Fslash", "escaped%2F"), Some("slash"));

    // Test with invalid percent encoding
    assert_eq!(TestServer::strip_prefix("/path%2rest", "/path"), Some("%2rest"));
    assert_eq!(TestServer::strip_prefix("/path%2rest", "/"), Some("path%2rest"));
    assert_eq!(TestServer::strip_prefix("/path%2Grest", "/path%2G"), None);

    // Test with incomplete Unicode code point
    assert_eq!(TestServer::strip_prefix("/path%D1rest", "/path%"), None);

    // Test with multi-byte characters
    assert_eq!(TestServer::strip_prefix("/a-%D1%BF/", "/a-ѿ"), Some("/"));
    assert_eq!(TestServer::strip_prefix("%D1%BF%25", "ѿ%"), Some(""));
  }

  #[test]
  fn test_strip_segment_suffix() {
    assert_eq!(TestServer::strip_segment_suffix("pre-suf/rest/of/path", "-suf"), Some(("pre", "/rest/of/path")));
    assert_eq!(TestServer::strip_segment_suffix("pre-suf", "-suf"), Some(("pre", "")));
    assert_eq!(TestServer::strip_segment_suffix("pre-suf/", "-suf"), Some(("pre", "/")));
    assert_eq!(TestServer::strip_segment_suffix("-suf", "-suf"), Some(("", "")));

    // Test no match
    assert_eq!(TestServer::strip_segment_suffix("random", "-suf"), None);
    assert_eq!(TestServer::strip_segment_suffix("random/", "-suf"), None);
    assert_eq!(TestServer::strip_segment_suffix("random/rest/of/path", "-suf"), None);

    // Matches & strips only the last occurence in the segment
    assert_eq!(TestServer::strip_segment_suffix("pre-suf-suf/rest", "-suf"), Some(("pre-suf", "/rest")));

    // With encoded characters
    assert_eq!(TestServer::strip_segment_suffix("some-%D1%BF-char/rest", "-ѿ-char"), Some(("some", "/rest")));
    assert_eq!(TestServer::strip_segment_suffix("%D1%BF", "ѿ"), Some(("", "")));
    assert_eq!(TestServer::strip_segment_suffix("%25%D1%BF", "ѿ"), Some(("%25", "")));

    // With encoded slash (special handling)
    assert_eq!(TestServer::strip_segment_suffix("escaped%2Fslash", "d%2Fslash"), Some(("escape", "")));

    // With invalid encoded characters
    assert_eq!(TestServer::strip_segment_suffix("some-%BF-char/rest", "-char"), Some(("some-%BF", "/rest")));
    assert_eq!(TestServer::strip_segment_suffix("some-%BF-char/rest", "%BF-char"), None);
  }
}
