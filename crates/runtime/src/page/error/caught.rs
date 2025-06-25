use std::any::TypeId;

/// A trait implemented by the type of the argument which is passed to the error loaders.
///
/// The implementor contains the type-erased version of an error thrown by some page loader.
///
/// This trait provides methods to downcast the contained error to a concrete type, as well
/// as to get some useful metadata, like the Rust-provided type name of the contained error,
/// or the identifier of the route where this error originated from.
pub trait Caught {
  /// Returns a reference to the contained error of type `T` if the error matches that type.
  /// Otherwise returns `None`.
  fn get_error<T: 'static>(&self) -> Option<&T>;

  /// Returns a mutable reference to the contained error of type `T` if the error matches that type.
  /// Otherwise returns `None`.
  ///
  /// Mutating the contained value will cause the subsequent calls to `get`, `get_mut` and `take`
  /// see the mutated value – no clones are made.
  fn get_error_mut<T: 'static>(&mut self) -> Option<&mut T>;

  /// Returns the contained error of type `T` if the error matches that type.
  /// Otherwise returns `None`.
  ///
  /// This method consumes the contained error on successful match, so the subsequent calls to `get`,
  /// `get_mut` and `take` will all return `None` after this method has yeielded `Some`.
  fn take_error<T: 'static>(&mut self) -> Option<T>;

  /// Returns some useful information about the origin of this error.
  fn thrown_by(&self) -> ThrownBy;

  /// Returns the type name of the contained error for diagnostic and logging purposes.
  ///
  /// The exact contents and format of the string returned are not specified, other than being a best-effort
  /// description of the type. For example, amongst the strings that `type_name::<Option<String>>()` might return
  /// are `"Option<String>"` and `"std::option::Option<std::string::String>"`.
  ///
  /// The returned string must not be considered to be a unique identifier of a type as multiple types may map
  /// to the same type name. Similarly, there is no guarantee that all parts of a type will appear in the returned
  /// string: for example, lifetime specifiers are currently not included. In addition, the output may change
  /// between versions of the Rust compiler.
  ///
  /// The current implementation uses the same infrastructure as compiler diagnostics and debuginfo, but this is
  /// not guaranteed.
  fn error_type_name(&self) -> &'static str;

  /// Returns the globally unique TypeId of the contained error for diagnostic purposes.
  fn error_type_id(&self) -> TypeId;
}

#[derive(Copy, Clone)]
pub struct ThrownBy {
  /// The identifier of the Route Segment where this error originated from.
  pub route_id: &'static str,
  /// The kind of the loader that threw this error.
  pub loader_kind: ThrownByLoaderKind,
}

#[derive(Copy, Clone)]
pub enum ThrownByLoaderKind {
  /// Indicates that the error was thrown by a Layout loader.
  Layout,
  /// Indicates that the error was thrown by a Page loader.
  Page,
  /// Indicates that the error was thrown by a NotFound Page loader.
  NotFoundPage,
  /// Indicates that the error was thrown by a downstream Error Page loader.
  ///
  /// When an error loader itself returns an error, Ruxy invokes the error loader of
  /// the closest parent route segment that has an error loader defined, passing the
  /// error to it.
  ///
  /// If the top-most error loader returns an error, Ruxy will return a generic HTTP
  /// response with status code 500 and body "Internal Server Error", logging a line
  /// to the stderr about an "uncaught" error.
  ErrorPage,
}
