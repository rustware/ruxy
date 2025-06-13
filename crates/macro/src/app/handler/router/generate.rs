use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_routing::{
  DynamicSequence, MatchDirection, RequestHandler, RouteSegment, RouteSequence, RouteSequenceMatcher, RouteTree,
  SegmentEffect, TrailingSlashConfig, TypedSequence,
};
use ::ruxy_util::radix_trie::RadixTrie;

use crate::app::config::AppConfig;

use super::context::GenContext;
use super::render::render_trie;
use super::wrappers;

type Trie = RadixTrie<TokenStream>;

pub fn generate(config: &AppConfig, routes: &RouteTree) -> TokenStream {
  // Create RouterContext to pass it to the nested generators
  let ctx = GenContext { config, routes };

  // Generate a Radix Trie recursively for all sequences from the root
  let radix_trie = create_radix_trie(&ctx, &routes.root_sequence);

  // Render the Radix Trie into a TokenStream (we always start with LTR matching)
  let radix_trie = render_trie(&radix_trie, MatchDirection::Ltr);

  // Generate a global 404 handler
  let global_404 = gen_global_404();

  quote! {
    #radix_trie
    #global_404
  }
}

fn create_radix_trie(ctx: &GenContext, sequence: &RouteSequence) -> Trie {
  // Get the sequence's containing segment
  let segment = &ctx.routes.segments[&sequence.containing_segment_id];

  // Create a new Radix Trie for each segment
  let mut trie = RadixTrie::new();

  // Extend the trie with all the children tries
  for child in &sequence.children {
    // Create Radix Trie for the child sequence
    let child_trie = create_radix_trie(ctx, child);

    // Extend the current segment's trie with the child trie
    trie.extend(child_trie);
  }

  let route_handler_segment = sequence.concludes_segment_id.as_ref().map(|id| &ctx.routes.segments[id]);

  // Insert the sequences's handler into the trie if it has one
  if let Some(handler) = route_handler_segment.and_then(|s| s.route_handler.as_ref()) {
    let is_root = is_root_sequence(ctx, sequence);

    let key = match (is_root, &ctx.config.trailing_slash) {
      (true, TrailingSlashConfig::RequireAbsent) => "/",
      (true, TrailingSlashConfig::RedirectToRemoved) => "/",
      _ => "",
    };

    let target = gen_segment_responder(ctx, segment, handler);
    let target = quote! { if path.is_empty() { #target } };

    trie.insert(key, target);
  }

  // At this point, the trie contains all children tries, and this segment's handler.
  // Now, depending on the segment's effect, we will prefix, wrap, or return the trie intact.
  //
  // "Wrapping" means returning a new trie containing a single TokenStream item with the subtrie rendered in it.
  // "Prefixing" means returning the same trie, but with a prefix added to all paths.
  //
  // It depends on each sequence's effect to decide whether the returned trie is constructed by
  // wrapping, prefixing, or some combination of both, or by returning the received subtrie intact.
  match &sequence.matcher {
    // TODO: This might need to take flipping into consideration
    RouteSequenceMatcher::Slash => trie.with_prefix("/"),
    RouteSequenceMatcher::Literal(literal) => trie.with_prefix(literal),
    RouteSequenceMatcher::Dynamic(_) => wrappers::with_dynamic_sequence(ctx, sequence, trie),
    RouteSequenceMatcher::Custom => wrappers::with_custom_match(ctx, sequence, trie),
    // Other sequence types return the trie intact
    _ => trie,
  }
}

fn is_root_sequence(ctx: &GenContext, sequence: &RouteSequence) -> bool {
  let Some(segment) = ctx.routes.segments.get(&sequence.containing_segment_id) else {
    return false;
  };

  segment.is_root
}

fn gen_segment_responder(ctx: &GenContext, segment: &RouteSegment, handler: &RequestHandler) -> TokenStream {
  let identifier = &segment.identifier;

  let path_params: Vec<TokenStream> = extract_idents_for_segment(segment, ctx.routes);

  quote! {
    let mut response = hyper::Response::builder();

    response = response.status(200);
    response = response.header("Content-Type", "text/html");

    let mut body = internal::ResponseBody::new();

    body.push(internal::Bytes::from("<!DOCTYPE html>"));
    body.push(internal::Bytes::from("<html>"));
    body.push(internal::Bytes::from("<head>"));
    body.push(internal::Bytes::from("<meta charset=\"utf-8\" />"));
    body.push(internal::Bytes::from("</head>"));
    body.push(internal::Bytes::from("<body>"));
    body.push(internal::Bytes::from("<div>Matched handler:</div>"));
    body.push(internal::Bytes::from("<div style=\"color: red;\">"));
    body.push(internal::Bytes::from(#identifier));
    body.push(internal::Bytes::from("</div>"));
    body.push(internal::Bytes::from("<div style=\"margin-top: 16px;\">Path params:</div>"));
    body.push(internal::Bytes::from("<div style=\"color: darkgreen;\">"));
    #(#path_params)*
    body.push(internal::Bytes::from("</div>"));
    body.push(internal::Bytes::from("</body>"));
    body.push(internal::Bytes::from("</html>"));

    return internal::HandlerResult {
      response: response.body(body)
    };
  }
}

fn extract_idents_for_segment(segment: &RouteSegment, routes: &RouteTree) -> Vec<TokenStream> {
  let mut v = Vec::new();

  if let SegmentEffect::UrlMatcher { sequences } = &segment.effect {
    let dyn_var_name = sequences.iter().find_map(|s| {
      let TypedSequence::Dynamic(DynamicSequence { param_name, .. }) = &s.typed else {
        return None;
      };

      Some(param_name)
    });

    if let Some(dyn_var_name) = dyn_var_name {
      let dyn_var_ident = format!("path_param_{}", segment.hex);
      let dyn_var_ident = Ident::new(&dyn_var_ident, Span::mixed_site());

      v.push(quote! {
        body.push(internal::Bytes::from("<div>"));
        let formatted = format!("{}: {:?}", #dyn_var_name, #dyn_var_ident);
        body.push(internal::Bytes::from(formatted));
        body.push(internal::Bytes::from("</div>"));
      });
    }
  }

  if let Some(parent) = &segment.parent {
    if let Some(parent) = routes.segments.get(parent) {
      v.extend(extract_idents_for_segment(parent, routes))
    }
  }

  v
}

fn gen_global_404() -> TokenStream {
  quote! {
    let mut response = hyper::Response::builder();

    response = response.status(404);
    response = response.header("Content-Type", "text/plain");

    let mut body = internal::ResponseBody::new();

    body.push(internal::Bytes::from("Not Found"));

    internal::HandlerResult {
      response: response.body(body)
    }
  }
}
