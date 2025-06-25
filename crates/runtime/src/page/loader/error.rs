use std::any::TypeId;

pub trait LoaderError: 'static {
  fn is<T: 'static>(&self) -> bool {
    std::any::Any::type_id(self) == TypeId::of::<T>()
  }
  
  fn get<T>(&self) -> Option<T> {
    todo!()
  }
}
