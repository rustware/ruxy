use bytes::Bytes;
use hyper::body::{Frame, SizeHint};
use std::fmt::{Display, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct ResponseBody {
  bytes: Vec<Bytes>,
  tail_cursor: usize,
  total_bytes: usize,
}

impl ResponseBody {
  pub fn new() -> Self {
    Self { bytes: Vec::new(), tail_cursor: 0, total_bytes: 0 }
  }

  pub fn push(&mut self, bytes: Bytes) {
    self.total_bytes += bytes.len();
    self.bytes.push(bytes);
  }
}

#[derive(Debug)]
pub struct ResponseBodyError;

impl Display for ResponseBodyError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Response body error occurred")
  }
}

impl std::error::Error for ResponseBodyError {}

impl hyper::body::Body for ResponseBody {
  type Data = Bytes;
  type Error = ResponseBodyError;

  fn poll_frame(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
    let index = self.tail_cursor;

    let Some(bytes) = self.bytes.get_mut(index) else {
      return Poll::Ready(None);
    };

    let bytes = std::mem::take(bytes);
    self.tail_cursor += 1;
    Poll::Ready(Some(Ok(Frame::data(bytes))))
  }

  fn is_end_stream(&self) -> bool {
    self.tail_cursor == self.bytes.len()
  }

  fn size_hint(&self) -> SizeHint {
    SizeHint::with_exact(self.total_bytes as u64)
  }
}
