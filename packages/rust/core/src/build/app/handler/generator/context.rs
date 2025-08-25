use crate::routing::routary::Routary;

pub struct GenContext<'a> {
  pub routes: &'a Routary,
}
