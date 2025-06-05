mod build_segments;
mod resolve_segment_effect;

use std::collections::HashMap;
use std::path;
use quote::quote;
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
  /// A effect of this segment to the routing and "special" behavior (slots, ...)
  pub effect: SegmentEffect,
  /// Unique HEX-encoded identifier of this segment.
  /// This can be used as a unique identifier in generated code.
  pub hex: String,
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

/// A Segment Effect determines how this Route Segment affects routing.
#[derive(Debug)]
pub enum SegmentEffect {
  /// This can be a Route Group (either aliased – `(x)`, or fully expressed – `{var[0]}`).
  /// Segment with a Group effect does NOT consume any URL segment and ALWAYS matches.
  Group,
  /// This can "branch" the matching into multiple different streams, rendered into multiple
  /// different slots in this segment's page. The user code will contain a `<Slot name="x" />`.
  /// Slots have their own separate URLs trees (enabling true parallel routing) which gets
  /// encoded into URL as query parameters `@<slot name>=<encoded url>`. This means when the
  /// user refreshes the page, all the slots will get rendered exactly what they've seen before.
  /// This also enables sharing URLs to the exact current state of the whole page.
  /// Segment with a Slot effect does NOT consume any URL segment and ALWAYS matches.
  /// Segments nested inside a Slot can consume URL segments as per their own effects, but they
  /// consume their URL contained in their own query parameter and NOT the main URL.
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
  /// Matches empty URL segment (the segment between `foo` and `bar` in `/foo//bar`).
  /// Directory name to match this segment is `_`.
  EmptySegment,
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

impl DynamicSequenceArity {
  pub fn get_rust_type(&self) -> proc_macro2::TokenStream {
    match self {
      Self::Exact(num) => match num {
        1 => quote! { String },
        other => quote! { [String; #other] },
      },
      Self::Range(lower, ..) => match lower {
        0 => quote! { Vec<String> },
        lower => quote! { ([String; #lower], Vec<String>) },
      },
    }
  }
}

impl SegmentEffect {
  /// Optional segments are those with lower arity bound set to 0.
  pub fn is_optional(&self) -> bool {
    match self {
      SegmentEffect::Group => false,
      SegmentEffect::Slot { .. } => false,
      SegmentEffect::UrlMatcher { .. } => {
        let Some(DynamicSequenceArity::Range(lower, ..)) = self.get_dynamic_sequence_arity() else {
          return true;
        };
        
        *lower > 0
      },
      SegmentEffect::EmptySegment => true,
    }
  }

  pub fn get_dynamic_sequence_arity(&self) -> Option<&DynamicSequenceArity> {
    match self {
      SegmentEffect::UrlMatcher { sequences } => {
        sequences.iter().find_map(|seq| {
          if let UrlMatcherSequence::Dynamic { arity, .. } = seq {
            Some(arity)
          } else {
            None
          }
        })
      },
      _ => None,
    }
  }
}

impl UrlMatcherSequence {
  /// Returns Some(literal) if this sequence is a literal sequence.
  /// Otherwise returns None.
  pub fn get_literal(&self) -> Option<&String> {
    match self {
      UrlMatcherSequence::Literal(literal) => Some(literal),
      _ => None
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
      },
      _ => None,
    }
  }
  
  pub fn is_dynamic(&self) -> bool {
    match &self.effect {
      SegmentEffect::UrlMatcher { sequences } => {
        if let Some(sequence) = sequences.first() {
          return sequence.get_literal().is_none()
        }
        
        sequences.len() > 1
      },
      _ => false,
    }
  }
}
