use std::path;

pub fn get_routes_dir() -> path::PathBuf {
  let Ok(crate_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
    panic!(
      "CARGO_MANIFEST_DIR is not set. If you're not using Cargo, please provide this path manually."
    );
  };
  
  path::PathBuf::from(crate_dir).join("src").join("routes")
}
