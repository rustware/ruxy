use crate::segment::Arity;
use crate::sequence::MatchDirection;

mod create_instructions;
mod inflate_instructions;
mod instructors;

#[derive(Debug)]
pub struct MatchInstruction {
  kind: InstructionKind,
  next: Vec<MatchInstruction>
}

impl Default for MatchInstruction {
  fn default() -> Self {
    Self {
      kind: InstructionKind::Skip,
      next: vec![],
    }
  }
}

impl PartialEq for MatchInstruction {
  fn eq(&self, other: &Self) -> bool {
    self.kind == other.kind
  }
}

#[derive(Debug, PartialEq)]
pub enum InstructionKind {
  /// Jump to the next instruction while ignoring the current one.
  Skip,
  /// Create a separate variable holding a string slice of the path.
  /// This will then be used for matching in this isolated slice.
  CreateView(CreateViewInstruction),
  /// The .0 is the name of the parameter to capture.
  /// This should be as simple as `let var = &path[..]`
  CaptureRestOfPath(String),
  /// The .0 is the count of the segments to consume.
  /// The .1 is the character length constraints that must be checked within each segment.
  ConsumeSegmentCount(usize, Arity),
  /// The .0 is the upper limit of the segments to consume.
  /// The .1 is the character length constraints that must be checked within each segment.
  ConsumeUpToSegmentCount(usize, Arity),
  /// Consume all remaining segments from the path.
  /// The .0 is the character length constraints that must be checked within each segment.
  ConsumeAllSegments(Arity),
  /// Invoke a user-specified matcher.
  /// The .0 is the ID of the Route Segment that contains the matcher.
  InvokeCustomMatcher(String),
  /// The .0 is the name of the parameter to capture.
  CaptureParam(String, MatchTarget),
  /// Consume a part of the path, .0 is the literal
  ConsumeLiteral(String, MatchDirection),
  /// Check if the URL is at the end (beware of special handling of slash in root segment).
  CheckEndOfUrl,
  /// Respond with the handler of the provided Segment ID
  InvokeRouteHandler(String),
  /// Respond with the Not Found handler of the provided Segment ID.
  /// This should be the last child in the series of children instructions.
  InvokeNotFoundHandler(String),
}

/// View is some range of URL characters (e.g. from end to 3 slashes left, exclusive)
#[derive(Debug, PartialEq)]
pub struct CreateViewInstruction {
  
}

#[derive(Debug, PartialEq)]
pub struct MatchTarget {
  kind: TargetKind,
  direction: MatchDirection,
  skip_slash_count: usize,
}

#[derive(Debug, PartialEq)]
pub struct ConsumeInstruction {
  
}

#[derive(Debug, PartialEq)]
pub enum TargetKind {
  ExactChars(usize),
  RestOfPath,
  RestOfView,
}
