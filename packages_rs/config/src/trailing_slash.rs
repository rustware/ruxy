#[derive(Default)]
pub enum TrailingSlashConfig {
  /// Requires that the trailing slash is present in the request URL.
  ///
  /// If the trailing slash is missing, a 404 response will be returned.
  RequirePresent,
  /// Requires that the trailing slash is NOT present in the request URL.
  ///
  /// If the trailing slash is present, a 404 response will be returned.
  RequireAbsent,
  /// Allows for the trailing slash to be present in the request URL.
  ///
  /// If the trailing slash is present, it is ignored and the request
  /// is routed as if the trailing slash was not present in the URL.
  ///
  /// If using this option, you will probably want to set the canonical
  /// URL for one of the URL versions – either with or without slash –,
  /// so that the crawling engines don't penalize you.
  ///
  /// Note that this setting prevents you from matching Empty Segments as leaf
  /// segments, as well as matching leaf Dynamic Segments to an empty value,
  /// because the trailing slash otherwise required for these matchers to match
  /// are ignored.
  ///
  /// Examples of unreachable or partially unreachable segments:
  /// ```text
  /// routes/
  /// └── nested/
  ///     ├─── _/
  ///     │   └── page.rs
  ///     └─── {param(0..)}/
  ///         └── page.rs
  /// ```
  /// In this case, neither of the leaf routes will match the request with URL `/nested/`,
  /// because the trailing slash is ignored. `{param(0..)}` segment would however still match `/nested/foo`.
  ///
  /// For this reason, it is recommended that your leaf Dynamic Segments are either defined
  /// with a prefix and/or a suffix, or that you remove the character length specifier allowing
  /// segment with an empty value to be matched – e.g. change it to `(1..)`, which is a default.
  ///
  /// For the same reason, it is also recommended to avoid leaf Empty Segment matchers `_`,
  /// as they cannot be matched at all when this option is active.
  ///
  /// If you don't follow these recommendations, you will get a compile-time warning about
  /// having non-reachable or partially non-reachable routes.
  Ignore,
  /// Requests containing a trailing slash in the URL are redirected to
  /// the same URL but with the trailing slash removed.
  ///
  /// This is the default behavior when no `trailing_slash` config is specified.
  ///
  /// Note that this setting prevents you from matching Empty Segments as leaf
  /// segments, as well as matching leaf Dynamic Segments to an empty value,
  /// because the user will be redirected to a different URL even when these
  /// would otherwise match.
  ///
  /// Examples of unreachable or partially unreachable segments:
  /// ```text
  /// routes/
  /// └── nested/
  ///     ├─── _/
  ///     │   └── page.rs
  ///     └─── {param(0..)}/
  ///         └── page.rs
  /// ```
  /// In this case, neither of the leaf routes will match the request with URL `/nested/`,
  /// because it is redirected. `{param(0..)}` segment would however still match `/nested/foo`.
  ///
  /// For this reason, it is recommended that your leaf Dynamic Segments are either defined
  /// with a prefix and/or a suffix, or that you remove the character length specifier allowing
  /// segment with an empty value to be matched – e.g. change it to `(1..)`, which is a default.
  ///
  /// For the same reason, it is also recommended to avoid leaf Empty Segment matchers `_`,
  /// as they cannot be matched at all when this option is active.
  ///
  /// If you don't follow these recommendations, you will get a compile-time warning about
  /// having non-reachable or partially non-reachable routes.
  #[default]
  RedirectToRemoved,
  /// Requests that do not contain a trailing slash in the URL are redirected
  /// to the same URL but with the trailing slash added.
  RedirectToAdded,
}

impl TrailingSlashConfig {
  pub fn get_routing_prefix(&self) -> &'static str {
    match self {
      Self::RequireAbsent | Self::RedirectToRemoved => "/",
      _ => "",
    }
  }
}
