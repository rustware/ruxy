use std::path::PathBuf;

pub fn get_project_dir() -> PathBuf {
  let Some(crate_dir) = std::env::var_os("CARGO_MANIFEST_DIR") else {
    panic!(
      "CARGO_MANIFEST_DIR is not set.\
      If you're not using Cargo, please provide this path manually."
    );
  };
  
  let Ok(project_dir) = PathBuf::from(crate_dir).canonicalize() else {
    panic!(
      "CARGO_MANIFEST_DIR is not set to a readable directory.\
      If you're not using Cargo, please provide a correct path."
    );
  };
  
  project_dir
}
