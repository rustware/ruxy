mod render;

use proc_macro2::TokenStream;
use quote::quote;

use crate::routing::instruction::MatchInstruction;
use crate::build::app::context::GenContext;
use render::render_instruction;

pub fn gen_matcher(ctx: &GenContext) -> TokenStream {
  // Generate rendered instructions recursively from the root
  render_instruction_recursive(ctx, &ctx.routary.root_match_instruction)
}

fn render_instruction_recursive(ctx: &GenContext, instruction: &MatchInstruction) -> TokenStream {
  let children = instruction.next.iter().map(|i| render_instruction_recursive(ctx, i));
  let children = quote! { #(#children)* };

  render_instruction(ctx, &instruction.kind, children)
}

