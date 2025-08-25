use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote};

use crate::build::app::context::GenContext;
use crate::routing::segment::{RenderTarget, RouteSegment};

pub fn gen_loader_call(ctx: &GenContext, segment: &RouteSegment, target: &RenderTarget) -> TokenStream {
  let Some(rs_module) = &target.rs_module else {
    return quote! {};
  };

  let module_name = Ident::new(&rs_module.name, Span::mixed_site());

  quote! {
    let loaded = #module_name::inner::loader().await;
  }
}
