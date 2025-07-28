#[derive(Debug, Clone, Copy)]
pub enum CrateType {
  Library,
  Binary,
}

impl CrateType {
  pub fn current() -> CrateType {
    match std::env::var_os("CARGO_BIN_NAME") {
      Some(_) => CrateType::Binary,
      None => CrateType::Library,
    }
  }
  
  pub fn is_binary(&self) -> bool {
    match self {
      CrateType::Binary => true,
      CrateType::Library => false,
    }
  }

  pub fn is_library(&self) -> bool {
    !self.is_binary()
  }
}

impl Default for CrateType {
  fn default() -> Self {
    CrateType::current()
  }
}
