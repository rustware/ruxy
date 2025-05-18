use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;

use ::ruxy_routing::{RouteTree, RouteSegment, RouteSegmentFileModule, RequestHandler};

pub fn gen_module_declarations(route_tree: &RouteTree) -> TokenStream {
  let declarations = route_tree.segments.values().map(gen_module_declarations_for_segment);
  quote! { #(#declarations)* }
}

pub fn gen_module_declarations_for_segment(segment: &RouteSegment) -> TokenStream {
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
    let path = &module.path;

    let ident = Ident::new(&module.name, Span::call_site());

    quote! {
      #[path = #path]
      mod #ident;
    }
  });

  quote! { #(#declarations)* }
}
