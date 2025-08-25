use crate::build::BuildConfig;
use crate::routing::routary::Routary;

pub struct GenContext<'a> {
  pub build_config: &'a BuildConfig,
  pub routary: &'a Routary,
}
