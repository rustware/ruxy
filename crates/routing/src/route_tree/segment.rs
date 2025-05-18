mod build_segments;
mod resolve_segment_role;

use std::collections::HashMap;
use std::path;

pub use build_segments::*;

/// Route Segment represents a single directory nested any number of levels deep inside the "routes" directory,
/// provided that this directory contains either:
/// - one of the valid Route Handler files (`page.rs` or `handler.rs`),
/// - at least one nested Route Segment directory.
///
/// In this structure, *all* directories are Route Segments:
/// ```text
/// routes/
/// ├── page.rs
/// └── deeply/
///     └── nested/
///         └── directory/
///             └── page.rs
/// ```
///
/// In this structure, only `routes` and `nested1` are Route Segments, other directories are ignored:
/// ```text
/// routes/
/// ├── page.rs
/// ├── nested1/
/// │   └── page.rs
/// └── nested2/
///     ├── nested3/
///     └── random.txt
/// ```
#[derive(Debug)]
pub struct RouteSegment {
  /// The directory name of this segment
  pub dir_name: String,
  /// Absolute path to this segment's directory
  pub fs_abs_path: path::PathBuf,
  /// Relative path from the "routes" directory
  pub fs_rel_path: path::PathBuf,
  /// Relative path from the "routes" directory (String)
  pub identifier: SegmentIdentifier,
  /// Only the root segment has None as its parent
  pub parent: Option<SegmentIdentifier>,
  /// Option containing Route Handler config for this segment,
  /// `None` if this segment does not have a Route Handler.
  pub route_handler: Option<RequestHandler>,
  /// Option containing Not Found Handler config for this segment,
  /// `None` if this segment does not have a Not Found Handler.
  pub not_found_handler: Option<RequestHandler>,
  /// Option containing Error Handler config for this segment,
  /// `None` if this segment does not have a Not Found Handler.
  pub error_handler: Option<RequestHandler>,
  /// Option containing Layout module info for this segment,
  /// `None` if this segment does not have a Not Found Handler.
  pub layout_module: Option<RouteSegmentFileModule>,
  /// Whether this segment is a leaf segment (i.e. it does not have any nested segments)
  pub is_leaf: bool,
  /// Whether this segment is a root segment (i.e. it is the root of the "routes" directory)
  pub is_root: bool,
  /// A role of this segment in relation to routing and "special" behavior (slots, ...)
  pub role: SegmentRole,
}

/// Segment identifier is its relative path from the "routes" directory.
/// This is an empty string for the root segment ("routes" directory).
pub type SegmentIdentifier = String;

/// Request Handler for this Route Segment. This determines which file
/// will be declared as a module and called for request processing.
#[derive(Debug)]
pub enum RequestHandler {
  /// The page handler is a file named `<prefix>page.rs`.
  /// A presence of `<prefix>(^page\.(?:j|t)sx?$)` is required.
  /// `<prefix>` is either `""`, `"not_found_"` or `"error_"`.
  Page { module: RouteSegmentFileModule },
  /// The custom handler is a file named `<prefix>handler.rs`.
  /// `<prefix>` is either `""`, `"not_found_"` or `"error_"`.
  Custom { module: RouteSegmentFileModule },
}

/// A module for file to be declared and used in imports.
/// This will be turned into a module declaration at the top of `main`.
#[derive(Debug)]
pub struct RouteSegmentFileModule {
  pub name: String,
  pub path: String,
}

/// A flat map of all the Route Segments
pub type SegmentMap = HashMap<SegmentIdentifier, RouteSegment>;

/// A segment Role determines its behavior as part of the routing tree.
#[derive(Debug)]
pub enum SegmentRole {
  /// This can be a Route Group (either aliased – `(x)`, or fully expressed – `{var[0]}`).
  PassThrough,
  /// This can "branch" the matching into multiple different streams, rendered into multiple
  /// different slots in the resulting page. The user code will contain a `<Slot name="x" />`.
  Slot {
    /// The name of this slot. To be used in `<Slot name="..." />`
    name: String,
  },
  UrlMatcher {
    /// Sequences are used to create the radix trie by the `app!` macro to match against URLs.
    /// Sequences represent the segment sequences like `prefix-` (literal) and `{var}` (dynamic),
    /// respecting their order of appearance in the directory name. Every sequence matcher must
    /// either "consume" a part of URL and move the matching forward, or "divert" and end processing
    /// of its routing branch.
    sequences: Vec<UrlMatcherSequence>,
  },
}

#[derive(Debug)]
pub enum UrlMatcherSequence {
  Literal(String),
  Dynamic { var_name: String, arity: DynamicSequenceArity },
}

/// Arity determines the number of URL segments to be matched by a dynamic sequence
#[derive(Debug)]
pub enum DynamicSequenceArity {
  /// Exact number of URL segments to be matched by this dynamic sequence
  Exact(usize),
  /// Range is inclusive on both sides
  Range(usize, Option<usize>),
}
