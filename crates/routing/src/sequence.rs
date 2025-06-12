mod build_sequence_tree;

use crate::{Arity, DynamicSequence};

pub use build_sequence_tree::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RouteSequenceMatcher {
  Root,
  Slash,
  Literal(String),
  Dynamic(DynamicSequence),
  Custom,
}

#[derive(Debug, Clone)]
pub struct RouteSequence {
  /// The index of this sequence inside its route segment.
  pub containing_segment_id: String,
  /// Matching behavior of this sequence.
  pub matcher: RouteSequenceMatcher,
  /// Direction in which we'll match against the URL pathname.
  pub url_path_direction: MatchDirection,
  /// Direction in which we'll match against the URL path segment.
  pub url_segment_direction: MatchDirection,
  /// Starting sequence of the Route Segment.
  pub is_segment_start: bool,
  /// Ending sequence of the Route Segment.
  pub is_segment_end: bool,
  /// Route Sequences that are nested inside this one.
  pub children: Vec<RouteSequence>,
}

impl RouteSequence {
  pub fn is_seg_count_range(&self) -> bool {
    matches!(self.matcher, RouteSequenceMatcher::Dynamic(DynamicSequence { seg_count: Arity::Range { .. }, .. }))
  }
  
  pub fn is_char_len_range(&self) -> bool {
    matches!(self.matcher, RouteSequenceMatcher::Dynamic(DynamicSequence { char_len: Arity::Range { .. }, .. }))
  }
}

impl PartialEq for RouteSequence {
  fn eq(&self, other: &Self) -> bool {
    self.containing_segment_id == other.containing_segment_id
      && self.matcher == other.matcher
      && self.url_path_direction == other.url_path_direction
      && self.is_segment_start == other.is_segment_start
      && self.is_segment_end == other.is_segment_end
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatchDirection {
  /// Match against the URL in left-to-right direction.
  Ltr,
  /// Match against the URL in right-to-left direction.
  Rtl,
}
