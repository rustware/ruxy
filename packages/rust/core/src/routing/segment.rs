mod build_segment_map;
mod create_segment_id;
mod file_registry;
mod get_route_segments;
mod parse_segment;

use std::collections::HashMap;
use std::path::PathBuf;
pub use build_segment_map::*;
pub use create_segment_id::*;
pub use file_registry::*;
pub use get_route_segments::*;
pub use parse_segment::*;

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
  /// Relative path from the "routes" directory (String)
  pub identifier: SegmentIdentifier,
  /// Identifier of the parent segment.
  /// Only the root segment has None as its parent.
  pub parent: Option<SegmentIdentifier>,
  /// A vector of error messages if this segment couldn't be parsed successfully.
  /// We still want to include its modules in the `app!` macro output, but we don't want
  /// to allow compiling the user application until the raised issue is addressed.
  pub compile_errors: Vec<String>,
  /// Identifiers of direct descendant segments
  pub children: Vec<SegmentIdentifier>,
  /// Option containing Route Target config for this segment,
  /// `None` if this segment does not have a Route Target.
  pub route_target: Option<EitherTarget>,
  /// Option containing Not Found Target config for this segment,
  /// `None` if this segment does not have a Not Found Handler.
  pub not_found_target: Option<EitherTarget>,
  /// Option containing Error Target config for this segment,
  /// `None` if this segment does not have an Error Target.
  pub error_target: Option<MultiTarget>,
  /// Option containing Layout Target config for this segment,
  /// `None` if this segment does not have a Layout Target.
  pub layout_target: Option<RenderTarget>,
  /// Whether this segment is a leaf segment (i.e. it does not have any nested segments)
  pub is_leaf: bool,
  /// Whether this segment is a root segment (i.e. it is the root of the "routes" directory)
  pub is_root: bool,
  /// A routing effect of this segment, or a "special" behavior (slots, fragments, custom matchers, ...).
  pub effect: SegmentEffect,
  /// Unique HEX-encoded identifier of this segment.
  /// This can be used as a unique identifier in generated code.
  pub hex: String,
}

/// Segment identifier is its relative path from the "routes" directory.
/// This is an empty string for the root segment ("routes" directory).
pub type SegmentIdentifier = String;

/// A Render Target is composed of a Rust module and a client entry.
/// A presence of a client entry is required.
///
/// If the `client_entry` is `None`, the behavior depends on the build mode:
/// - For Development builds, Ruxy will generate a default client entry with explainer of how to fix the situation.
/// - For Production builds, the build will fail with an error explaining the situation.
#[derive(Debug)]
pub struct RenderTarget {
  pub rs_module: Option<RouteSegmentRsModule>,
  pub client_entry: Option<RouteSegmentClientEntry>
}

/// A Handler Target is a Rust module responsible for producing a response.
///
/// Handler targets are raw responders written by the user.
/// It can be used for API routes, various kinds of streaming services, proxies, etc.
#[derive(Debug)]
pub struct HandlerTarget {
  pub rs_module: RouteSegmentRsModule,
}

/// A target that can be either a Render Target or a Handler Target.
#[derive(Debug)]
pub enum EitherTarget {
  Render(RenderTarget),
  Handler(HandlerTarget),
}

/// A target containing both a Render Target and a Handler Target.
/// As an example, this is used for Error Targets, which can be
/// invoked based on the type of the RouteTarget that errored.
#[derive(Debug, Default)]
pub struct MultiTarget {
  pub render: Option<RenderTarget>,
  pub handler: Option<HandlerTarget>,
}

/// A module for file to be declared and used in imports.
/// This will be turned into a module declaration at the top of `main`.
#[derive(Debug)]
pub struct RouteSegmentRsModule {
  pub name: String,
  pub path: String,
}

/// A client entry file (js/jsx/ts/tsx/md/mdx).
#[derive(Debug)]
pub struct RouteSegmentClientEntry {
  /// File name (e.g. "page.tsx")
  pub name: String,
  /// File path from routes directory (e.g. "foo/bar/page.tsx")
  pub path: PathBuf,
  pub ext: RouteSegmentClientEntryExt,
}

#[derive(Debug)]
pub enum RouteSegmentClientEntryExt {
  Js,
  Jsx,
  Ts,
  Tsx,
  Md,
  Mdx,
}

/// A flat map of all the Route Segments
pub type SegmentMap = HashMap<SegmentIdentifier, RouteSegment>;

/// A Segment Effect determines how this Route Segment affects routing.
#[derive(Debug)]
pub enum SegmentEffect {
  /// This is a Route Group (either aliased – `(x)`, or fully expressed – `{var[0]}`).
  /// Segment with a Group effect does NOT consume any URL segment and ALWAYS matches.
  Group,
  /// This can "branch" the matching into multiple different streams, rendered into multiple
  /// different slots in this segment's page. The user code will contain a `<Slot name="x" />`.
  ///
  /// Slots have their own separate URLs trees (enabling true parallel routing) which gets
  /// encoded into URL as query parameters `@<slot name>=<encoded url>`. This means when the
  /// user refreshes the page, all the slots will get rendered exactly what they've seen before.
  /// This also enables sharing URLs to the exact current state of the whole page.
  ///
  /// Segment with a Slot effect does NOT consume any URL segment and ALWAYS matches.
  ///
  /// Segments nested inside a Slot can consume URL segments as per their own effects, but they
  /// consume their URL contained in their own query parameter and NOT the primary URL.
  Slot {
    /// The name of this slot. To be used in `<Slot name="..." />`
    name: String,
  },
  /// URL matcher can either match or divert.
  /// If it matches, it consumes 0 or more URL segments.
  /// If it diverts, it MUST NOT consume any URL segments.
  UrlMatcher {
    /// Sequences are used to create the radix trie by the `app!` macro to match against URLs.
    /// Sequences represent the segment sequences like `prefix-` (literal) and `{var}` (dynamic),
    /// respecting their order of appearance in the directory name.
    sequences: Vec<UrlMatcherSequence>,
  },
  /// A Custom Match segment is a segment prefixed with `~` in its directory name, containing
  /// a file named `match.rs` from which the user will export their own custom matching function.
  CustomMatch {
    /// The identifier of this Custom Match segment. Will be present in the custom path params
    /// struct passed into subsequent layouts and Route Handlers.
    identifier: String,
  },
  /// Matches empty URL segment (the segment between `foo` and `bar` in `/foo//bar`).
  /// Directory name to match this segment is `_`.
  EmptySegment,
}

#[derive(Debug)]
pub struct UrlMatcherSequence {
  /// The index of the start position of this sequence in the directory name
  pub start_pos: usize,
  pub typed: TypedSequence,
}

#[derive(Debug)]
pub enum TypedSequence {
  Literal(String),
  Dynamic(DynamicSequence),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DynamicSequence {
  /// The name of the path parameter as defined by the user.
  pub param_name: String,
  /// The number of URL segments to match.
  pub seg_count: Arity,
  /// The number of characters to match.
  pub char_len: Arity,
  /// Whether this sequence is the first sequence of the Route Segment.
  pub is_first: bool,
  /// Whether this sequence is the last sequence of the Route Segment.
  pub is_last: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SplitMatchMergeType {
  /// Prepend the separated value to the rest of the values.
  Prepend,
  /// Append the separated value to the rest of the values.
  Append,
}

impl Default for DynamicSequence {
  fn default() -> Self {
    Self {
      param_name: "".to_string(),
      seg_count: Arity::Exact(1),
      char_len: Arity::Range(1, None),
      is_first: false,
      is_last: false,
    }
  }
}

/// A configuration determining the number of items to match by a dynamic sequence.
/// An "item" can be either an URL Segment, or a character.
///
/// This is used for specifying the Segment Count and Character Length of a dynamic sequence.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Arity {
  /// Exact count of items to match.
  Exact(usize),
  /// Range is inclusive on both sides.
  Range(usize, Option<usize>),
}

impl Arity {
  pub fn get_min(&self) -> usize {
    match self {
      Self::Exact(exact) => *exact,
      Self::Range(min, _) => *min,
    }
  }

  pub fn get_max(&self) -> Option<usize> {
    match self {
      Self::Exact(exact) => Some(*exact),
      Self::Range(_, max) => *max,
    }
  }
}

impl UrlMatcherSequence {
  /// Returns Some(literal) if this sequence is a literal sequence.
  /// Otherwise returns None.
  pub fn get_literal(&self) -> Option<&String> {
    match self {
      UrlMatcherSequence { typed: TypedSequence::Literal(literal), .. } => Some(literal),
      _ => None,
    }
  }
}

impl RouteSegment {
  /// Returns Some(literal) if this segment is a UrlMatcher segment with
  /// a single literal sequence. Otherwise returns None.
  pub fn get_literal(&self) -> Option<&String> {
    match &self.effect {
      SegmentEffect::UrlMatcher { sequences } => {
        if sequences.len() > 1 {
          return None;
        }

        sequences.first()?.get_literal()
      }
      _ => None,
    }
  }
}
