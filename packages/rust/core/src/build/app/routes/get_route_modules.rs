use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::routing::routary::Routary;
use crate::routing::segment::{EitherTarget, RouteSegment, RouteSegmentRsModule, RenderTarget, HandlerTarget, MultiTarget};
use crate::util::fs::get_project_dir;

pub fn gen_route_modules(route_tree: &Routary) -> TokenStream {
  let declarations = route_tree.segment_map.values().map(gen_route_modules_for_segment);
  quote! { #(#declarations)* }
}

pub fn gen_route_modules_for_segment(segment: &RouteSegment) -> TokenStream {
  let mut modules = Vec::<&RouteSegmentRsModule>::new();

  macro_rules! extract_module_from_target {
    ($prop:ident) => {
      if let Some(target) = &segment.$prop {
        match target {
          EitherTarget::Render(RenderTarget { rs_module: Some(m), .. }) => modules.push(m),
          EitherTarget::Handler(HandlerTarget { rs_module, .. }) => modules.push(rs_module),
          _ => {},
        };
      }
    };
  }

  extract_module_from_target!(route_target);
  extract_module_from_target!(not_found_target);

  if let Some(target) = &segment.error_target {
    if let Some(HandlerTarget { rs_module, .. }) = &target.handler {
      modules.push(rs_module);
    }

    if let Some(RenderTarget { rs_module: Some(module), .. }) = &target.render {
      modules.push(module);
    }
  }

  if let Some(RenderTarget { rs_module: Some(module), .. }) = &segment.layout_target {
    modules.push(module);
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
