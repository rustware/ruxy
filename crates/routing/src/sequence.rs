mod get_route_sequences;
mod get_segment_sequences;

use crate::segment::{Arity, DynamicSequence};

pub use get_route_sequences::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RouteSequence {
  Root,
  Slash,
  Literal(String),
  Dynamic(DynamicSequence),
  /// The String is the ID of the Route Segment holding this custom matcher.
  Custom(String),
}

impl RouteSequence {
  pub fn is_seg_count_range(&self) -> bool {
    matches!(self, RouteSequence::Dynamic(DynamicSequence { seg_count: Arity::Range { .. }, .. }))
  }

  pub fn is_char_len_range(&self) -> bool {
    matches!(self, RouteSequence::Dynamic(DynamicSequence { char_len: Arity::Range { .. }, .. }))
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatchDirection {
  /// Match in left-to-right direction.
  Ltr,
  /// Match in right-to-left direction.
  Rtl,
}
