use std::collections::HashMap;

pub struct Headers {
  map: HashMap<String, String>,
}

impl std::ops::Deref for Headers {
  type Target = HashMap<String, String>;

  fn deref(&self) -> &Self::Target {
    &self.map
  }
}

impl std::ops::DerefMut for Headers {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.map
  }
}
