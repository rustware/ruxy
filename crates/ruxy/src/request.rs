pub mod headers;
pub mod cookies;

pub struct Request {
  pub headers: headers::Headers,
  pub cookies: cookies::Cookies,
}
