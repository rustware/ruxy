use std::error::Error;

use crate::page::Props;
use crate::redirect::Redirect;

pub type LoaderResult<P = ()> = Result<LoaderOutput<P>, Box<dyn Error>>;

pub struct LoaderOutput<P: Props = ()> {
  data: LoaderData<P>,
}

enum LoaderData<P: Props> {
  Props(P),
  Redirect(Redirect),
}

impl<P: Props> From<Redirect> for LoaderOutput<P> {
  fn from(value: Redirect) -> Self {
    LoaderOutput { data: LoaderData::Redirect(value) }
  }
}

impl<P: Props> From<Redirect> for LoaderResult<P> {
  fn from(value: Redirect) -> Self {
    Ok(value.into())
  }
}
