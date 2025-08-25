/// A parsed configuration for the build process.
/// Mostly parsed from command-line arguments or cfg attributes.
pub struct BuildConfig {
  pub mode: BuildMode,
}

pub enum BuildMode {
  Development,
  Production,
}

impl BuildConfig {
  pub fn parse() -> Self {
    Self {
      mode: if cfg!(debug_assertions) { BuildMode::Development } else { BuildMode::Production },
    }
  }
}
