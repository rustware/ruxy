use std::path::Path;

pub fn main() {
  let serialized = emit_watch_hints_for_dir(Path::new("src/routes"));
  let hashed = short_hash(&serialized);
  
  // Updates `.ruxy/ROUTES_HASH` with a build tag
  update_routes_hash_file(format!("{:x}", hashed));
}

fn emit_watch_hints_for_dir(path: &Path) -> String {
  let mut file_names = Vec::new();

  if let Ok(entries) = std::fs::read_dir(path) {
    for entry in entries.flatten() {
      let path = entry.path();

      if path.is_file() {
        let name = path.file_name().unwrap_or_default();
        let name = name.to_str().unwrap_or_default();
        file_names.push(name.to_string());
      }
      
      if !path.is_dir() {
        continue;
      }

      if let Some(path) = path.to_str() {
        println!("cargo::rerun-if-changed={}", path);
      }

      let nested = emit_watch_hints_for_dir(&path);
      file_names.push(nested);
    }
  }

  let dirname = path.file_name().unwrap_or_default();
  let dirname = dirname.to_str().unwrap_or_default();
  
  format!("{dirname}({})", file_names.join("|"))
}

fn update_routes_hash_file(hash: String) {
  let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") else {
    panic!("CARGO_MANIFEST_DIR is not set. Are you running this from cargo?");
  };

  let manifest_dir = std::path::PathBuf::from(manifest_dir);
  let ruxy_cache_dir = manifest_dir.join(".ruxy");

  std::fs::create_dir(&ruxy_cache_dir).unwrap_or_else(|err| {
    if err.kind() != std::io::ErrorKind::AlreadyExists {
      panic!("Could not create the `.ruxy` directory: {}", err);
    }
  });

  let routes_hash_file_path = manifest_dir.join(".ruxy/ROUTES_HASH");
  std::fs::write(routes_hash_file_path, hash).unwrap();
}

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

fn short_hash(input: &str) -> u64 {
  let mut hasher = DefaultHasher::new();
  input.hash(&mut hasher);
  hasher.finish()
}
