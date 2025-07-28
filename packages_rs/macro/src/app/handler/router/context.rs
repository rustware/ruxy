use ::ruxy_routing::route_tree::RouteTree;

use crate::app::input::AppMacroInput;

pub(super) struct GenContext<'a> {
  pub input: &'a AppMacroInput,
  pub routes: &'a RouteTree,
}
