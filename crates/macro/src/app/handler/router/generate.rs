mod global_404;
mod responder;

use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::instruction::MatchInstruction;
use ::ruxy_routing::route_tree::RouteTree;

use crate::app::config::MacroConfig;
use crate::app::handler::router::generate::global_404::gen_global_404;
use crate::app::handler::router::render::render_instruction;

use super::context::GenContext;

pub(super) use responder::gen_segment_responder;

pub fn generate(config: &MacroConfig, routes: &RouteTree) -> TokenStream {
  // Create GenContext to pass it to the nested generators
  let ctx = GenContext { config, routes };

  // Generate rendered instructions recursively from the root
  let instructions = render_instruction_recursive(&ctx, &routes.root_instruction);

  // Generate a global 404 handler
  let global_404 = gen_global_404();

  quote! {
    #instructions
    #global_404
  }
}

fn render_instruction_recursive(ctx: &GenContext, instruction: &MatchInstruction) -> TokenStream {
  let children = instruction.next.iter().map(|i| render_instruction_recursive(ctx, i));
  let children = quote! { #(#children)* };

  render_instruction(ctx, &instruction.kind, children)
}

// type Trie = RadixTrie<TokenStream>;
//
// pub fn generate_old(routes: &RouteTree) -> TokenStream {
//   // Create RouterContext to pass it to the nested generators
//   let ctx = GenContext { config: &APP_CONFIG, routes };
//
//   // Generate a Radix Trie recursively for all sequences from the root
//   let radix_trie = create_radix_trie(&ctx, &routes.root_instruction);
//
//   // Render the Radix Trie into a TokenStream (we always start with LTR matching)
//   let radix_trie = render_trie(&radix_trie, MatchDirection::Ltr);
//
//   // Generate a global 404 handler
//   let global_404 = gen_global_404();
//
//   quote! {
//     #radix_trie
//     #global_404
//   }
// }
//
// fn create_radix_trie(ctx: &GenContext, sequence: &MatchInstruction) -> Trie {
//   // Get the sequence's containing segment
//   let segment = &ctx.routes.segments[&sequence.containing_segment_id];
//
//   // Create a new Radix Trie for each segment
//   let mut trie = RadixTrie::new();
//
//   // Extend the trie with all the children tries
//   for child in &sequence.children {
//     // Create Radix Trie for the child sequence
//     let child_trie = create_radix_trie(ctx, child);
//
//     // Extend the current segment's trie with the child trie
//     trie.extend(child_trie);
//   }
//
//   let route_handler_segment = sequence.concludes_segment_id.as_ref().map(|id| &ctx.routes.segments[id]);
//
//   // Insert the sequences's handler into the trie if it has one
//   if let Some(handler) = route_handler_segment.and_then(|s| s.route_handler.as_ref()) {
//     let is_root = is_root_sequence(ctx, sequence);
//
//     let key = match (is_root, &ctx.config.trailing_slash) {
//       (true, TrailingSlashConfig::RequireAbsent) => "/",
//       (true, TrailingSlashConfig::RedirectToRemoved) => "/",
//       _ => "",
//     };
//
//     let target = gen_segment_responder(ctx, segment, handler);
//     let target = quote! { if path.is_empty() { #target } };
//
//     trie.insert(key, target);
//   }
//
//   // At this point, the trie contains all children tries, and this segment's handler.
//   // Now, depending on the segment's effect, we will prefix, wrap, or return the trie intact.
//   //
//   // "Wrapping" means returning a new trie containing a single TokenStream item with the subtrie rendered in it.
//   // "Prefixing" means returning the same trie, but with a prefix added to all paths.
//   //
//   // It depends on each sequence's effect to decide whether the returned trie is constructed by
//   // wrapping, prefixing, or some combination of both, or by returning the received subtrie intact.
//   match &sequence.matcher {
//     // TODO: This might need to take flipping into consideration
//     RouteSequenceMatcher::Slash => trie.with_prefix("/"),
//     RouteSequenceMatcher::Literal(literal) => trie.with_prefix(literal),
//     RouteSequenceMatcher::Dynamic(_) => wrappers::with_dynamic_sequence(ctx, sequence, trie),
//     RouteSequenceMatcher::Custom => wrappers::with_custom_match(ctx, sequence, trie),
//     // Other sequence types return the trie intact
//     _ => trie,
//   }
// }
//
// fn is_root_sequence(ctx: &GenContext, sequence: &RouteSequence) -> bool {
//   let Some(segment) = ctx.routes.segments.get(&sequence.containing_segment_id) else {
//     return false;
//   };
//
//   segment.is_root
// }
