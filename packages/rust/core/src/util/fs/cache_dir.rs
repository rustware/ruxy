use std::path::PathBuf;

use super::get_ruxy_out_dir;

pub fn get_cache_dir() -> PathBuf {
  let cache_dir = get_ruxy_out_dir().join("cache");

  std::fs::create_dir(&cache_dir).unwrap_or_else(|err| {
    if err.kind() != std::io::ErrorKind::AlreadyExists {
      panic!("Could not create the `.ruxy/cache` directory: {err}");
    }
  });

  cache_dir
}
