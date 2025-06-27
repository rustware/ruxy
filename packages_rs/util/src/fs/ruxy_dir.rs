use std::path::PathBuf;

use super::get_out_dir;

pub fn get_ruxy_dir() -> PathBuf {
  let ruxy_dir = get_out_dir().join(".ruxy");

  std::fs::create_dir(&ruxy_dir).unwrap_or_else(|err| {
    if err.kind() != std::io::ErrorKind::AlreadyExists {
      panic!("Could not create the `.ruxy` directory: {err}");
    }
  });

  ruxy_dir
}
