use crate::routing::segment::Arity;

pub mod create_instructions;
mod inflate_instructions;
mod instructors;
mod validators;

/// A single route-matching instruction.
/// Used for generating Rust and JS code for matching pathnames to routes.
#[derive(Debug)]
pub struct MatchInstruction {
  pub kind: MatchInstructionKind,
  /// Nested instructions to execute if the current instruction matches.
  /// Note: Some instructions (like 
  pub next: Vec<MatchInstruction>,
}

impl Default for MatchInstruction {
  fn default() -> Self {
    Self { kind: MatchInstructionKind::Skip, next: vec![] }
  }
}

impl PartialEq for MatchInstruction {
  fn eq(&self, other: &Self) -> bool {
    self.kind == other.kind
  }
}

#[derive(Debug, PartialEq)]
pub enum MatchInstructionKind {
  /// Jump to the next instruction while ignoring the current one.
  /// Used as a root instruction and a default.
  Skip,
  /// Create a separate variable initialized with a string slice of the path, where the string slice
  /// is holding the rest of an URL segment (up until a slash or end of URL) in the given direction.
  /// This will then be used for matching inside this isolated slice.
  /// The .2 is the character offset to exclude from the view on its end (or start if RTL).
  /// Don't forget to consume the contents of the view from the path.
  ConsumeIntoView(MatchDirection, usize),
  /// The .0 is the name of the parameter to capture.
  /// This should be as simple as `let var = &path[..]`
  CaptureRestOfPath(String),
  /// The .0 is the count of the segments to consume.
  /// The .1 is the character length constraints that must be checked within each segment.
  ConsumeSegmentCount(usize, Arity, MatchDirection),
  /// The .0 is the upper limit of the segments to consume.
  /// The .1 is the character length constraints that must be checked within each segment.
  /// Direction is always LTR, and even the first segment is preceded by slash.
  ConsumeUpToSegmentCount(usize, Arity),
  /// Consume all remaining segments from the path.
  /// The .0 is the character length constraints that must be checked within each segment.
  /// Direction is always LTR, and even the first segment is preceded by slash.
  ConsumeAllSegments(Arity),
  /// Check if the path is empty or if the first character is a slash.
  /// If the first character is a slash, consume it.
  /// ```rs
  /// if let Some(path) = path.strip_prefix('/').or_else(|| if path.is_empty() { Some("") } else { None }) {}
  /// ```
  PathEmptyOrConsumeSlash,
  /// The .0 is the name of the parameter to capture.
  /// The .1 is the exact number of characters to capture.
  CaptureExactChars(String, usize, MatchDirection),
  /// The .0 is the name of the parameter to capture.
  /// The .1 is the exact number of characters to capture from the view.
  CaptureExactCharsInView(String, usize, MatchDirection),
  /// The .0 is is the exact number of characters to consume from the view.
  ConsumeExactCharsInView(usize, MatchDirection),
  /// Capture the rest of the characters in the view.
  /// The .0 is the name of the parameter to capture.
  CaptureRestOfView(String),
  /// Checks whether the rest of the view has the length between the given bounds.
  CheckCharLenInRestOfView(usize, Option<usize>),
  /// Invoke a user-specified matcher.
  /// The .0 is the ID of the Route Segment that contains the matcher.
  InvokeCustomMatcher(String),
  /// Consume a part of the path, .0 is the literal.
  ConsumeLiteral(String, MatchDirection),
  /// Consume a part of the view, .0 is the literal.
  ConsumeLiteralInView(String, MatchDirection),
  /// Check if the path is at the end.
  CheckEndOfPath,
  /// Respond with the handler of the provided Segment ID
  InvokeRouteHandler(String),
  /// Respond with the Not Found handler of the provided Segment ID.
  /// This should be the last child in the series of children instructions.
  InvokeNotFoundHandler(String),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatchDirection {
  /// Match in left-to-right direction.
  Ltr,
  /// Match in right-to-left direction.
  Rtl,
}
