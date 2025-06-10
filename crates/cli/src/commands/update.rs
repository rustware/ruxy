use std::path::PathBuf;

pub struct CmdUpdateOptions {
  pub directory: Option<PathBuf>,
}

pub fn cmd_update(mut options: CmdUpdateOptions) {
  // TODO:
  //  - Invalidate the routes hash (echo "[INVALIDATED:<random-string>]" -> .ruxy/ROUTES_HASH
  //  - Re-generate the TS types for routes
}
