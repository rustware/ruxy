pub mod cookies;
pub mod headers;
pub mod path_params;

use super::request::path_params::{PathParameters, UntypedPathParams};

pub struct Request<PathParams: PathParameters = UntypedPathParams> {
  pub headers: headers::Headers,
  pub cookies: cookies::Cookies,
  pub path_params: PathParams,
}
