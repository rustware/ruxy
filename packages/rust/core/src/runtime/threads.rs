use std::num::NonZeroUsize;

const ENV_THREADS: &str = "RUXY_THREADS";

pub fn get_thread_count() -> usize {
  match std::env::var(ENV_THREADS) {
    Err(std::env::VarError::NotUnicode(e)) => {
      panic!("\"{ENV_THREADS}\" must be valid unicode, error: {e:?}")
    }
    Err(std::env::VarError::NotPresent) => {
      std::thread::available_parallelism().map_or(1, NonZeroUsize::get)
    }
    Ok(value) => {
      let parsed = value.parse().unwrap_or_else(|e| {
        panic!("\"{ENV_THREADS}\" must be a valid number, error: {e}, value: {value}")
      });
      
      assert!(parsed > 0, "\"{ENV_THREADS}\" cannot be set to 0");
      
      parsed
    }
  }
}
