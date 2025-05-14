use std::collections::HashMap;

pub struct Cookie {
  name: String,
  value: String,
  domain: String,
  path: String,
  expires: std::time::Instant,
  max_age: std::time::Duration,
  secure: bool,
  http_only: bool,
}

pub struct Cookies {
  map: HashMap<String, Cookie>,
}

impl std::ops::Deref for Cookies {
  type Target = HashMap<String, Cookie>;

  fn deref(&self) -> &Self::Target {
    &self.map
  }
}

impl std::ops::DerefMut for Cookies {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.map
  }
}

