use std::path::Path;

pub fn emit_watch_hints_for_dir(path: &Path) {
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

      emit_watch_hints_for_dir(&path);
    }
  }
}