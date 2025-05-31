mod route_tree;

pub use route_tree::*;

pub enum TrailingSlashConfig {
  RequirePresent,
  RequireAbsent,
  Ignore,
}
