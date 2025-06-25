use crate::page::Props;
use crate::page::loader::LoaderResult;

/// The user can choose one of multiple return types for their loader function.
/// All types that implement this trait can be used as such.
///
/// Ruxy uses this trait internally to convert the return value of the loader function
/// into a value of normalized (unified) type, as well as to statically validate whether
/// the chosen return type is allowed or not.
pub trait LoaderReturn<P: Props, E> {
  fn into_result(self) -> LoaderResult<P, E>;
}

// Allows returning any type that implements Into<LoaderResult<P, E>>
impl<P, E, T> LoaderReturn<P, E> for T
where
  P: Props,
  T: Into<LoaderResult<P, E>>,
{
  fn into_result(self) -> LoaderResult<P, E> {
    LoaderResult::from(self.into())
  }
}

// Allows returning Props directly
// impl<P: Props, E> LoaderReturn<P, E> for P {
//   fn into_result(self) -> LoaderResult<P, E> {
//     todo!()
//   }
// }
