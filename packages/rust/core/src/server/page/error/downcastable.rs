use std::any::TypeId;

use super::caught::{Caught, ThrownBy};

pub(crate) struct Downcastable<'err> {
  pub(crate) error: &'err mut dyn std::any::Any,
  pub(crate) thrown_by: ThrownBy,
  pub(crate) type_name: &'static str,
  pub(crate) type_id: TypeId,
}

impl<'err> Caught for Downcastable<'err> {
  fn get_error<T: 'static>(&self) -> Option<&T> {
    self.error.downcast_ref::<Option<T>>()?.as_ref()
  }

  fn get_error_mut<T: 'static>(&mut self) -> Option<&mut T> {
    self.error.downcast_mut::<Option<T>>()?.as_mut()
  }

  fn take_error<T: 'static>(&mut self) -> Option<T> {
    self.error.downcast_mut::<Option<T>>().and_then(std::mem::take)
  }
  
  fn thrown_by(&self) -> ThrownBy {
    self.thrown_by
  }

  fn error_type_name(&self) -> &'static str {
    self.type_name
  }

  fn error_type_id(&self) -> TypeId {
    self.type_id
  }
}
