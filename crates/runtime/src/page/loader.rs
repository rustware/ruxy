mod error;
mod loader_return;
mod output;

pub use output::LoaderOutput;

use crate::page::Props;
use crate::redirect;
use crate::redirect::Redirect;

/// Anything that implements this trait can be returned from a loader.
pub trait Loader: Into<Result<LoaderOutput<Self::Props>, Self::Error>> {
  type Props: Props;
  type Error;
}

/// Fully specified loader result type.
/// Everything else that can be returned from the loaders must be convertable to this type.
type LoaderResult<P, E> = Result<LoaderOutput<P>, E>;

impl<P: Props, E> From<LoaderOutput<P>> for LoaderResult<P, E> {
  fn from(value: LoaderOutput<P>) -> Self {
    Ok(value)
  }
}

impl<P: Props, E> From<Redirect> for LoaderResult<P, E> {
  fn from(value: Redirect) -> Self {
    Ok(value.into())
  }
}

impl Loader for Redirect {
  type Props = ();
  type Error = ();
}

async fn loader() -> impl Loader {
  redirect("/somewhere").permanent()
}

async fn macrogenerated() {
  let result: Result<_, _> = loader().await.into();

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
  // TODO: Check whether this the type_name is actually useful, otherwise try `type_name_of_val(error)`
  let downcastable = DowncastableErr { error: &mut Some(error), type_name: std::any::type_name::<E>() };
  let output = error_loader(downcastable).await;
}

trait Caught {
  fn get<T: 'static>(&self) -> Option<&T>;
  fn get_mut<T: 'static>(&mut self) -> Option<&mut T>;
  fn take<T: 'static>(&mut self) -> Option<T>;
  fn type_name(&self) -> &'static str;
}

struct DowncastableErr<'err> {
  error: &'err mut dyn ::std::any::Any,
  type_name: &'static str,
}

impl<'err> Caught for DowncastableErr<'err> {
  fn get<T: 'static>(&self) -> Option<&T> {
    self.error.downcast_ref::<Option<T>>()?.as_ref()
  }

  fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
    self.error.downcast_mut::<Option<T>>()?.as_mut()
  }

  fn take<T: 'static>(&mut self) -> Option<T> {
    self.error.downcast_mut::<Option<T>>().and_then(std::mem::take)
  }

  fn type_name(&self) -> &'static str {
    self.type_name
  }
}

struct MyErr {
  my_attr: String,
}

struct MyOtherErr {
  my_other_attr: String,
}

async fn error_loader(mut error: impl Caught) -> impl Loader {
  if let Some(err) = error.get::<MyErr>() {
    println!("{}", err.my_attr);
  }

  if let Some(err) = error.get_mut::<MyErr>() {
    err.my_attr.push('!');
    println!("{}", err.my_attr);
  }

  if let Some(mut err) = error.take::<MyErr>() {
    err.my_attr = "".to_string();
    println!("{}", err.my_attr);
  }

  if let Some(MyOtherErr { my_other_attr }) = error.get() {
    println!("{my_other_attr}");
  }

  if let Some(MyOtherErr { my_other_attr }) = error.get_mut() {
    my_other_attr.push('!');
    println!("{my_other_attr}");
  }

  if let Some(MyOtherErr { mut my_other_attr }) = error.take() {
    my_other_attr.push('!');
    println!("{my_other_attr}");
  }

  redirect("/somewhere").permanent()
}
