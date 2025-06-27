use std::collections::VecDeque;

use ::ruxy_config::{APP_CONFIG, TrailingSlashConfig};

use crate::instruction::inflate_instructions::inflate_instructions;
use crate::instruction::instructors::{instruct_dynamic_sequence, instruct_seg_count_range};
use crate::instruction::{InstructionKind, MatchDirection, MatchInstruction};
use crate::instruction::validators::non_ambiguity::validate_non_ambiguity;
use crate::segment::{RouteSegment, SegmentMap};
use crate::sequence::{RouteSequence, get_route_sequences};

pub fn create_instructions(segments: &SegmentMap) -> Result<MatchInstruction, Vec<String>> {
  let route_leaves = segments.values().filter_map(|s| {
    s.route_handler.as_ref()?;
    Some(s)
  });
  
  let mut routes = vec![];
  let mut errors = vec![];
  
  for handler_segment in route_leaves {
    match get_route_sequences(segments, handler_segment) {
      Ok(sequences) => routes.push((sequences, handler_segment)),
      Err(errs) => errors.extend(errs),
    }
  }
  
  if let Err(errs) = validate_non_ambiguity(&routes) {
    errors.extend(errs);
  }

  if !errors.is_empty() {
    return Err(errors);
  }
  
  let routes = routes.into_iter().map(|(seqs, handler)| create_route_instructions(seqs, handler));
  
  Ok(inflate_instructions(routes.collect()))

  // TODO: Create a radix trie from MatchInstruction prefixes instead of string prefixes
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

  if ctx.sequences.is_empty() {
    // Root is special, we handle it separately
    return create_root_instructions(segment);
  }

  while !ctx.sequences.is_empty() {
    create_route_instructions_loop(&mut ctx);
  }

  ctx.instructions
}

fn create_root_instructions(segment: &RouteSegment) -> Vec<MatchInstruction> {
  let mut instructions = Vec::new();
  
  if matches!(APP_CONFIG.trailing_slash, TrailingSlashConfig::RequireAbsent | TrailingSlashConfig::RedirectToRemoved) {
    let kind = InstructionKind::ConsumeLiteral(String::from("/"), MatchDirection::Ltr);
    instructions.push(MatchInstruction { kind, ..Default::default() });
  }
  
  instructions.extend([
    MatchInstruction { kind: InstructionKind::CheckEndOfPath, ..Default::default() },
    MatchInstruction {
      kind: InstructionKind::InvokeRouteHandler(segment.identifier.clone()),
      ..Default::default()
    },
  ]);
  
  instructions
}

fn create_route_instructions_loop(ctx: &mut CreateInstructionsContext) {
  // Handle SegCount:Range
  if ctx.sequences[0].is_seg_count_range() {
    if ctx.sequences.len() > 1 && !ctx.path_rtl {
      return ctx.path_rtl = true;
    }

    if ctx.sequences.len() == 1 {
      // Match the remaining SegCount Range segment
      let sequence = ctx.sequences.pop_front().unwrap();
      return instruct_seg_count_range(ctx, sequence);
    }
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
      MatchInstruction { kind: InstructionKind::CheckEndOfPath, ..Default::default() },
      MatchInstruction {
        kind: InstructionKind::InvokeRouteHandler(ctx.route_segment_id.clone()),
        ..Default::default()
      },
    ]);
  }
}
