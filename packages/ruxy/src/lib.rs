pub use ruxy_macro::{build, generator, loader, main, Props};

// Request-related items
pub use ruxy_core::server::request::Request;
pub use ruxy_core::server::request::cookies::Cookies;
pub use ruxy_core::server::request::headers::Headers;

// Page-related items
pub use ruxy_core::server::page::error::{self, Caught};
pub use ruxy_core::server::page::{GeneratorOutput, Loadable, LoaderOutput, Props};

// Items usable in both Page and Handler
pub use ruxy_core::server::redirect;

// Config-related items
pub use ruxy_core::config::AppConfig;
pub use ruxy_core::config::TrailingSlashConfig;
pub use ruxy_core::config::PartytownConfig;

// Internal (called in macro expansions)
#[doc(hidden)]
pub mod __ruxy_macro_internal {
  pub use ruxy_core::server::re::*;
  pub use ruxy_core::config::register_app_config;
  pub use ruxy_core::build::build;
}
