use std::path::Path;

use crate::routing::routary::Routary;

#[test]
fn test_build_routes() {
  let routes_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sand/app/routes");
  let _routary = Routary::parse(&routes_dir);
  
  // TODO: Test the resulting tree
}
