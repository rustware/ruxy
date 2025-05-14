mod get_routes_dir;
mod get_segments;
mod segment;

use std::collections::HashMap;
use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::quote;

use self::get_routes_dir::*;
use self::get_segments::*;
use self::segment::RouteSegment;
use self::segment::SegmentIdentifier;
use self::segment::{RequestHandler, RouteSegmentFileModule};

/// A flat map holding all the Route Segments.
type SegmentMap = HashMap<SegmentIdentifier, RouteSegment>;

pub struct Routes {
  pub segments: SegmentMap,
}

impl Routes {
  pub fn create() -> Self {
    let routes_dir = get_routes_dir();
    let segments = get_segments(routes_dir, 0, None);

    Routes { segments }
  }

  pub fn get_module_declarations(&self) -> TokenStream {
    let declarations = self.segments.values().map(|route| self.get_module_declarations_for_segment(route));
    quote! { #(#declarations)* }
  }

  pub fn get_module_declarations_for_segment(&self, segment: &RouteSegment) -> TokenStream {
    let mut modules = Vec::<&RouteSegmentFileModule>::new();

    macro_rules! extract_module_from_handler {
      ($prop:ident) => {
        if let Some(ref handler) = segment.$prop {
          modules.push(match handler {
            RequestHandler::Page { module } => module,
            RequestHandler::Custom { module } => module,
          });
        }
      };
    }

    extract_module_from_handler!(route_handler);
    extract_module_from_handler!(error_handler);
    extract_module_from_handler!(not_found_handler);

    if let Some(layout) = &segment.layout_module {
      modules.push(layout);
    };

    let declarations = modules.iter().map(|module| {
      let name = &module.name;
      let path = &module.path;

      format!("#[path = \"{path}\"] mod {name};")
    });

    let declarations = declarations.collect::<Vec<_>>().join("\r\n");
    TokenStream::from_str(&declarations).unwrap_or(Default::default())
  }

  pub fn get_routing(&self) -> TokenStream {
    quote! {
      async fn route() {
        // TODO: Bunch of match statements here composing a radix trie
      }
    }
  }
}
