use ::ruxy_routing::RouteTree;

use crate::app::config::AppConfig;

pub(super) struct GenContext<'a> {
  pub config: &'a AppConfig,
  pub routes: &'a RouteTree,
}
