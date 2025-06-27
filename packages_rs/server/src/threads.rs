pub(crate) fn num_threads() -> usize {
  use std::num::NonZeroUsize;

  const ENV_WORKER_THREADS: &str = "RUXY_WORKER_THREADS";

  match std::env::var(ENV_WORKER_THREADS) {
    Ok(s) => {
      let n = s.parse().unwrap_or_else(|e| {
        panic!("\"{ENV_WORKER_THREADS}\" must be a valid number, error: {e}, value: {s}")
      });
      assert!(n > 0, "\"{ENV_WORKER_THREADS}\" cannot be set to 0");
      n
    }
    Err(std::env::VarError::NotPresent) => {
      std::thread::available_parallelism().map_or(1, NonZeroUsize::get)
    }
    Err(std::env::VarError::NotUnicode(e)) => {
      panic!("\"{ENV_WORKER_THREADS}\" must be valid unicode, error: {e:?}")
    }
  }
}
