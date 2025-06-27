use std::path::Path;

use crate::route_tree::RouteTree;

#[test]
fn test_build_routes() {
  let routes_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sand/src/routes");
  let _tree = RouteTree::new(&routes_dir);
  
  // TODO: Test the resulting tree
}
