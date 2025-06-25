use super::{LoaderOutput, Props, Redirect};

pub trait RuxyLoader {
  type Props: Props;
  type Error;

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error>;
}

// Allows `Redirect` to be returned directly
impl RuxyLoader for Redirect {
  type Props = ();
  type Error = ();

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    Ok(self.into())
  }
}

// Allows `Props` to be returned directly
impl<P: Props> RuxyLoader for P {
  type Props = P;
  type Error = ();

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    Ok(self.into())
  }
}

// Allows `LoaderOutput` to be returned directly
impl<P: Props> RuxyLoader for LoaderOutput<P> {
  type Props = P;
  type Error = ();

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    Ok(self)
  }
}

// Allows returning full result
impl<P: Props, E> RuxyLoader for Result<LoaderOutput<P>, E> {
  type Props = P;
  type Error = E;

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    self
  }
}
