use ruxy_routing::RouteTree;
use crate::app::config::AppConfig;

pub struct RouterContext<'a> {
  pub config: &'a AppConfig,
  pub routes: &'a RouteTree,
}
