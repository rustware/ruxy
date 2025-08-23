#[derive(Clone)]
pub struct AsyncExecutor;

impl<F> hyper::rt::Executor<F> for AsyncExecutor
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  fn execute(&self, fut: F) {
    tokio::task::spawn(fut);
  }
}
