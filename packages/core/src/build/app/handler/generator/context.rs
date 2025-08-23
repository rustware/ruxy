use crate::routing::route_tree::RouteTree;

pub struct GenContext<'a> {
  pub routes: &'a RouteTree,
}
