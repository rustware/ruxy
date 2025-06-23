use std::borrow::Cow;

pub fn redirect(destination: impl Into<Cow<'static, str>>) -> Redirect {
  Redirect { destination: destination.into(), ..Default::default() }
}

pub struct Redirect {
  pub(crate) destination: Cow<'static, str>,
  pub(crate) status_code: u16,
}

impl Redirect {
  pub fn new(destination: impl Into<Cow<'static, str>>, status: u16) -> Self {
    Self { destination: destination.into(), status_code: status }
  }
}

impl Default for Redirect {
  fn default() -> Self {
    Self { destination: Cow::Borrowed("/"), status_code: 307 }
  }
}

impl Redirect {
  /// Sets a 307 status code for this redirect, marking it a temporary redirect.
  /// This is the default.
  pub fn temporary(self) -> Self {
    Redirect { status_code: 307, ..self }
  }

  /// Sets a 308 status code for this redirect, marking it a permanent redirect.
  pub fn permanent(self) -> Self {
    Redirect { status_code: 308, ..self }
  }
}
