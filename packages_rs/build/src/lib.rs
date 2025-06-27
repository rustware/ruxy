use std::path::Path;

pub fn main() {
  let serialized = emit_watch_hints_for_dir(Path::new("src/routes"));
  let hashed = short_hash(&serialized);

  // Updates `<out>/.ruxy/cache/ROUTES_HASH` with an updated routes hash
  update_routes_hash_file(format!("{hashed:x}"));
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
        println!("cargo::rerun-if-changed={path}");
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
  let cache_dir = get_cache_dir();

  let routes_hash_file_path = cache_dir.join("ROUTES_HASH");
  std::fs::write(routes_hash_file_path, hash).unwrap();
}

use ruxy_util::fs::get_cache_dir;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn short_hash(input: &str) -> u64 {
  let mut hasher = DefaultHasher::new();
  input.hash(&mut hasher);
  hasher.finish()
}
