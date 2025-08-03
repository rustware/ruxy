use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_routing::instruction::MatchInstruction;
use ::ruxy_routing::route_tree::RouteTree;

use crate::app::handler::generator::context::GenContext;
use crate::app::handler::generator::render::render_instruction;

pub fn gen_router(routes: &RouteTree) -> TokenStream {
  // Create GenContext to pass it to the nested generators
  let ctx = GenContext { routes };

  // Generate rendered instructions recursively from the root
  render_instruction_recursive(&ctx, &routes.root_instruction)
}

fn render_instruction_recursive(ctx: &GenContext, instruction: &MatchInstruction) -> TokenStream {
  let children = instruction.next.iter().map(|i| render_instruction_recursive(ctx, i));
  let children = quote! { #(#children)* };

  render_instruction(ctx, &instruction.kind, children)
}

