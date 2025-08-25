use std::borrow::Cow;

use crate::server::not_found::NotFound;
use crate::server::page::Props;
use crate::server::redirect::Redirect;

/// The output of a page loader.
///
/// It can contain either Props or a Redirect, but not both.
/// It can also contain additional headers and cookies to be set for the HTTP response.
pub struct LoaderOutput<P: Props = ()> {
  data: LoaderData<P>,
  headers: Cow<'static, [(Cow<'static, str>, Cow<'static, str>)]>,
  cookies: Cow<'static, [(Cow<'static, str>, Cow<'static, str>)]>,
}

impl LoaderOutput {
  // TODO: Document this
  pub fn add_header(&mut self, name: &str, value: &str) -> Self {
    todo!()
  }

  // TODO: Document this
  pub fn add_cookie(&mut self, name: &str, value: &str) -> Self {
    todo!()
  }
}

impl<P: Props> Default for LoaderOutput<P> {
  fn default() -> Self {
    LoaderOutput { data: Default::default(), headers: Default::default(), cookies: Default::default() }
  }
}

impl<P: Props> From<Redirect> for LoaderOutput<P> {
  /// Creates `LoaderOutput` instance from the provided `Redirect` instance.
  ///
  /// You can attach additional headers or cookies after the instance is created.
  fn from(value: Redirect) -> Self {
    LoaderOutput { data: LoaderData::Redirect(value), ..Default::default() }
  }
}

impl<P: Props> From<NotFound> for LoaderOutput<P> {
  /// Creates `LoaderOutput` instance from the provided `NotFound` instance.
  ///
  /// You can attach additional headers or cookies after the instance is created.
  fn from(_: NotFound) -> Self {
    LoaderOutput { data: LoaderData::NotFound, ..Default::default() }
  }
}

impl<P: Props> From<P> for LoaderOutput<P> {
  /// Creates `LoaderOutput` instance from the provided `Props` instance.
  ///
  /// You can attach additional headers or cookies after the instance is created.
  fn from(props: P) -> Self {
    LoaderOutput { data: LoaderData::Props(props), ..Default::default() }
  }
}

enum LoaderData<P: Props> {
  Props(P),
  Redirect(Redirect),
  NotFound,
}

impl<P: Props> Default for LoaderData<P> {
  fn default() -> Self {
    Self::Redirect(Default::default())
  }
}
