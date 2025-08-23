use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::routing::route_tree::RouteTree;
use crate::routing::segment::{RequestHandler, RouteSegment, RouteSegmentFileModule};
use crate::util::fs::get_project_dir;

pub fn gen_route_modules(route_tree: &RouteTree) -> TokenStream {
  let declarations = route_tree.segments.values().map(gen_route_modules_for_segment);
  quote! { #(#declarations)* }
}

pub fn gen_route_modules_for_segment(segment: &RouteSegment) -> TokenStream {
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

  let project_dir = get_project_dir();
  
  let declarations = modules.iter().map(|module| {
    let path = project_dir.join("app").join(&module.path);
    let path = path.to_str().unwrap();

    let outer_mod_ident = Ident::new(&module.name, Span::mixed_site());
    let inner_mod_ident = Ident::new("inner", Span::mixed_site());

    quote! {
      #[doc(hidden)]
      #[path = ""]
      mod #outer_mod_ident {
        // TODO: Parse the route file from the `app!` macro and output some importable
        //       stuff here. E.g. `super::PathParams` will have different type inside
        //       each route module. Here we place the route-specific `type PathParams = ...`.
        //       This way, we won't even need macros inside route modules (#[ruxy::laoder] etc.).
        //       Everything will be generated here.
        
        // TODO: Parse all `pub mod` declarations inside the route module and re-export them
        //       as `crate::routes::some_ident` where `some_ident` is the name of the module.
        //       This will allow users to selectively export stuff from route modules, or even
        //       re-use loaders between route modules (like route A using loader from route B).
        
        struct Route;
        
        #[doc(hidden)]
        #[path = #path]
        #[allow(refining_impl_trait)]
        mod #inner_mod_ident;
      }
    }
  });

  quote! { #(#declarations)* }
}
