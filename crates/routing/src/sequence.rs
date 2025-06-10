mod build_sequence_tree;

use crate::{Arity, DynamicSequence};

pub use build_sequence_tree::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RouteSequenceMatcher {
  /// `None` is a virtual matcher that always matches without consuming any URL bytes.
  None,
  Slash,
  Literal(String),
  Dynamic(DynamicSequence),
  Custom,
}

#[derive(Debug, Clone)]
pub struct RouteSequence {
  /// The index of this sequence inside its route segment.
  pub containing_segment_id: String,
  pub matcher: RouteSequenceMatcher,
  pub direction: MatchDirection,
  pub is_last_in_segment: bool,
  pub children: Vec<RouteSequence>,
}

impl RouteSequence {
  pub fn is_seg_count_range(&self) -> bool {
    matches!(self.matcher, RouteSequenceMatcher::Dynamic(DynamicSequence { seg_count: Arity::Range { .. }, .. }))
  }
}

impl PartialEq for RouteSequence {
  fn eq(&self, other: &Self) -> bool {
    self.containing_segment_id == other.containing_segment_id
      && self.matcher == other.matcher
      && self.direction == other.direction
      && self.is_last_in_segment == other.is_last_in_segment
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatchDirection {
  /// Match against the URL in left-to-right direction.
  Ltr,
  /// Match against the URL in right-to-left direction.
  Rtl,
}
