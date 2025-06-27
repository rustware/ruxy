#[derive(Clone)]
pub struct AsyncExecutor;

impl<F> hyper::rt::Executor<F> for AsyncExecutor
where
  F: Future + 'static,
  F::Output: Send + 'static,
{
  fn execute(&self, fut: F) {
    tokio::task::spawn_local(fut);
  }
}
