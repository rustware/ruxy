mod internal;
mod output;

use crate::not_found::NotFound;
use crate::page::Props;
use crate::redirect;
use crate::redirect::Redirect;

use super::error::caught::{Caught, ThrownBy, ThrownByLoaderKind};
use super::error::downcastable::Downcastable;

pub use output::LoaderOutput;

/// Anything that implements this trait can be returned from a loader
pub trait Loadable: internal::RuxyLoadable + Sized {
  // This should only be called from macro expansions.
  // We don't guarantee the API stability of this method.
  #[doc(hidden)]
  fn __internal_into_result(self) -> Result<LoaderOutput<Self::Props>, Self::Error> {
    internal::RuxyLoadable::into_loader_result(self)
  }
}

impl<T: internal::RuxyLoadable> Loadable for T {}

impl<P: Props, E> From<LoaderOutput<P>> for Result<LoaderOutput<P>, E> {
  fn from(value: LoaderOutput<P>) -> Self {
    Ok(value)
  }
}

impl<P: Props, E> From<Redirect> for Result<LoaderOutput<P>, E> {
  fn from(value: Redirect) -> Self {
    Ok(value.into())
  }
}

impl<P: Props, E> From<NotFound> for Result<LoaderOutput<P>, E> {
  fn from(value: NotFound) -> Self {
    Ok(value.into())
  }
}

async fn loader() -> impl Loadable {
  redirect("/somewhere").permanent()
}

async fn macrogenerated() {
  let result: Result<_, _> = loader().await.__internal_into_result();

  match result {
    Err(err) => {
      // call error handler
      let output = handle_error_macrogenerated(err).await;
      // if error handler returns another error, we'll bubble it up the route tree,
      // (we call the nearest parent with error handler), last is generic 500 returned by Ruxy
    }
    Ok(output) => {}
  };
}

async fn handle_error_macrogenerated<E: 'static>(error: E) -> () {
  let thrown_by = ThrownBy { route_id: "routes/project/{project_id}/(proj)", loader_kind: ThrownByLoaderKind::Page };

  // TODO: Check whether this type_name is actually useful, otherwise try `type_name_of_val(error)`
  let downcastable = Downcastable {
    error: &mut Some(error),
    thrown_by,
    type_name: std::any::type_name::<E>(),
    type_id: std::any::TypeId::of::<E>(),
  };

  let output = error_loader(downcastable).await;
}

struct MyErr {
  my_attr: String,
}

struct MyOtherErr {
  my_other_attr: String,
}

async fn error_loader(mut error: impl Caught) -> impl Loadable {
  if let Some(err) = error.get_error::<MyErr>() {
    println!("{}", err.my_attr);
  }

  if let Some(err) = error.get_error_mut::<MyErr>() {
    err.my_attr.push('!');
    println!("{}", err.my_attr);
  }

  if let Some(mut err) = error.take_error::<MyErr>() {
    err.my_attr = "".to_string();
    println!("{}", err.my_attr);
  }

  if let Some(MyOtherErr { my_other_attr }) = error.get_error() {
    println!("{my_other_attr}");
  }

  if let Some(MyOtherErr { my_other_attr }) = error.get_error_mut() {
    my_other_attr.push('!');
    println!("{my_other_attr}");
  }

  if let Some(MyOtherErr { mut my_other_attr }) = error.take_error() {
    my_other_attr.push('!');
    println!("{my_other_attr}");
  }

  redirect("/somewhere").permanent()
}
