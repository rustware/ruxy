use std::path::Path;

use crate::routing::route_tree::RouteTree;

#[test]
fn test_build_routes() {
  let routes_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sand/app/routes");
  let _tree = RouteTree::new(&routes_dir);
  
  // TODO: Test the resulting tree
}
