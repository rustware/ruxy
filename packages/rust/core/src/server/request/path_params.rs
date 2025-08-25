mod path_param;

pub use path_param::PathParam;

pub trait PathParameters {
  /// Returns the value of a path parameter of the provided name.
  ///
  /// If used in a Layout Function, this can also be used to target downstream
  /// path parameters that occur _after_ the current layout's route segment.
  ///
  /// # Return Value
  /// - `Some(&PathParam)` if a path parameter with the specified name exists,
  ///   regardless of whether it occurs before, after, or as part of the current segment.
  /// - `None` otherwise.
  fn get(&self, name: &str) -> Option<&PathParam<'_>>;

  /// Get the value of a path parameter with the provided name.
  ///
  /// If used in a Layout Function, this can be used to target downstream
  /// path parameters that occur _after_ the current layout's route segment.
  ///
  /// # Panic
  /// This function will panic if the path parameter with the provided name does not exist
  /// for the current matched route. If you want to handle this case, use `get` instead.
  fn get_unchecked(&self, name: &str) -> &str;
}

pub struct UntypedPathParams {}

impl UntypedPathParams {
  pub(crate) fn new() -> Self {
    Self {}
  }
}

/// A PathParams implementation that is generic over routes.
///
/// This can be used to get path parameters without specifying
/// the concrete type of the parameters based on concrete route.
///
/// This is mainly to allow the `Request` struct to be passed around
/// without specifying typed PathParams of any concrete route.
impl PathParameters for UntypedPathParams {
  fn get(&self, name: &str) -> Option<&PathParam<'_>> {
    todo!()
  }

  fn get_unchecked(&self, name: &str) -> &str {
    todo!()
  }
}
