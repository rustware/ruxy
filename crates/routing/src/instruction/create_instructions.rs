use std::collections::VecDeque;

use crate::instruction::inflate_instructions::inflate_instructions;
use crate::instruction::{InstructionKind, MatchInstruction, MatchTarget, TargetKind};
use crate::instruction::instructors::{instruct_dynamic_sequence, instruct_seg_count_range};
use crate::segment::{Arity, DynamicSequence, RouteSegment, SegmentMap};
use crate::sequence::{MatchDirection, RouteSequence, get_route_sequences};

pub fn create_instructions(segments: &SegmentMap) -> MatchInstruction {
  let routes = segments.values().filter_map(|s| {
    s.route_handler.as_ref()?;
    let sequences = get_route_sequences(segments, s);
    Some(create_route_instructions(sequences, s))
  });

  inflate_instructions(routes.collect())
  
  // TODO: Create a RadixTrie from MatchInstruction prefixes instead of string prefixes
}

pub struct CreateInstructionsContext {
  pub instructions: Vec<MatchInstruction>,
  pub route_segment_id: String,
  pub sequences: VecDeque<RouteSequence>,
  pub path_rtl: bool,
}

pub fn create_route_instructions(sequences: Vec<RouteSequence>, segment: &RouteSegment) -> Vec<MatchInstruction> {
  let mut ctx = CreateInstructionsContext {
    sequences: VecDeque::from(sequences),
    instructions: Vec::new(),
    route_segment_id: segment.identifier.clone(),
    path_rtl: false,
  };

  while !ctx.sequences.is_empty() {
    create_route_instructions_loop(&mut ctx);
  }

  ctx.instructions
}

fn create_route_instructions_loop(ctx: &mut CreateInstructionsContext) {
  // We can create instructions that say e.g. "jump to the beginning of the 3rd segment from end of URL"
  // and we'll be _removing_ all processed sequences from the `sequences` vec. When the vec is empty,
  // we'll insert a final instruction ("check if the URL is empty and match the route").

  // Handle SegCount:Range
  if ctx.sequences[0].is_seg_count_range() {
    if ctx.sequences.len() > 1 {
      return ctx.path_rtl = true;
    }

    // Match the remaining SegCount Range segment
    let sequence = ctx.sequences.pop_front().unwrap();
    return instruct_seg_count_range(ctx, sequence);
  }

  let sequence = match ctx.path_rtl {
    true => ctx.sequences.pop_back().unwrap(),
    false => ctx.sequences.pop_front().unwrap(),
  };
  
  match sequence {
    RouteSequence::Slash => {
      let direction = if ctx.path_rtl { MatchDirection::Rtl } else { MatchDirection::Ltr };
      let kind = InstructionKind::ConsumeLiteral(String::from("/"), direction);
      ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
    }
    RouteSequence::Literal(literal) => {
      let direction = if ctx.path_rtl { MatchDirection::Rtl } else { MatchDirection::Ltr };
      let kind = InstructionKind::ConsumeLiteral(literal, direction);
      ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
    }
    RouteSequence::Custom(segment_id) => {
      let kind = InstructionKind::InvokeCustomMatcher(segment_id);
      ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
    }
    RouteSequence::Dynamic(dyn_seq) => {
      instruct_dynamic_sequence(ctx, dyn_seq);
    }
    _ => {}
  };

  // Handle end of route
  if ctx.sequences.is_empty() && !ctx.path_rtl {
    ctx.instructions.extend([
      MatchInstruction { kind: InstructionKind::CheckEndOfUrl, ..Default::default() },
      MatchInstruction {
        kind: InstructionKind::InvokeRouteHandler(ctx.route_segment_id.clone()),
        ..Default::default()
      },
    ]);
  }
}
