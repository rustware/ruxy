use ::ruxy_routing::route_tree::RouteTree;

use crate::app::config::MacroConfig;

pub(super) struct GenContext<'a> {
  pub config: &'a MacroConfig,
  pub routes: &'a RouteTree,
}
