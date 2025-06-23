use std::path::PathBuf;

pub fn get_out_dir() -> PathBuf {
  let Some(out_dir) = std::env::var_os("OUT_DIR") else {
    panic!(
      "OUT_DIR is not set.\
      If you're not using Cargo, please provide this path manually."
    );
  };
  
  let Ok(out_dir) = PathBuf::from(out_dir).canonicalize() else {
    panic!(
      "OUT_DIR is not set to a readable directory.\
      If you're not using Cargo, please provide a correct path."
    );
  };

  out_dir
}
