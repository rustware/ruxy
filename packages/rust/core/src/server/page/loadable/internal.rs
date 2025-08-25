use crate::server::not_found::NotFound;
use super::{LoaderOutput, Props, Redirect};

pub trait RuxyLoadable {
  type Props: Props;
  type Error;

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error>;
}

// Allows `Redirect` to be returned directly
impl RuxyLoadable for Redirect {
  type Props = ();
  type Error = ();

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    Ok(self.into())
  }
}

// Allows `NotFound` to be returned directly
impl RuxyLoadable for NotFound {
  type Props = ();
  type Error = ();

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    Ok(self.into())
  }
}

// Allows `Props` to be returned directly
impl<P: Props> RuxyLoadable for P {
  type Props = P;
  type Error = ();

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    Ok(self.into())
  }
}

// Allows `LoaderOutput` to be returned directly
impl<P: Props> RuxyLoadable for LoaderOutput<P> {
  type Props = P;
  type Error = ();

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    Ok(self)
  }
}

// Allows returning full result
impl<P: Props, E> RuxyLoadable for Result<LoaderOutput<P>, E> {
  type Props = P;
  type Error = E;

  fn into_loader_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    self
  }
}
