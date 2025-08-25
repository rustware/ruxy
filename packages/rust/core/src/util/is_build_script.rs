/// Whether the caller is running in a build script.
pub fn is_build_script() -> bool {
  std::env::var("OUT_DIR").is_ok()
}