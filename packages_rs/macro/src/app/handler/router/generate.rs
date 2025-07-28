mod global_404;
mod responder;

use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::instruction::MatchInstruction;
use ::ruxy_routing::route_tree::RouteTree;

use crate::app::handler::router::generate::global_404::gen_global_404;
use crate::app::handler::router::render::render_instruction;

use super::context::GenContext;

pub(super) use responder::gen_segment_responder;
use crate::app::input::AppMacroInput;

pub fn generate(config: &AppMacroInput, routes: &RouteTree) -> TokenStream {
  // Create GenContext to pass it to the nested generators
  let ctx = GenContext { input: config, routes };

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
