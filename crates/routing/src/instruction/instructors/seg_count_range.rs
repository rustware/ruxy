use crate::instruction::create_instructions::CreateInstructionsContext;
use crate::instruction::{InstructionKind, MatchDirection, MatchInstruction};
use crate::segment::{Arity, DynamicSequence};
use crate::sequence::RouteSequence;

pub fn instruct_seg_count_range(ctx: &mut CreateInstructionsContext, sequence: RouteSequence) {
  let RouteSequence::Dynamic(dyn_seq) = sequence else {
    unreachable!("Unexpected route sequence type");
  };

  let DynamicSequence { seg_count: Arity::Range(seg_min, seg_max), char_len, param_name, .. } = dyn_seq else {
    unreachable!("Unexpected route sequence type");
  };

  // The leading slash is captured too (e.g. "/first/second")
  //    ^ -> "/a": ["a"], "/a/": ["a", ""], "//a": ["", "a"], "/": [""], "": []
  //  This needs to be handled at runtime (when creating the iterator)
  let kind = InstructionKind::CaptureRestOfPath(param_name);
  ctx.instructions.push(MatchInstruction { kind, ..Default::default() });

  if seg_min > 0 {
    let kind = InstructionKind::ConsumeSegmentCount(seg_min, char_len, MatchDirection::Ltr);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  }

  if let Some(max) = seg_max.map(|max| max - seg_min) {
    let kind = InstructionKind::ConsumeUpToSegmentCount(max, char_len);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
    let kind = InstructionKind::CheckEndOfPath;
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  } else if !matches!(char_len, Arity::Range(0, None)) {
    // Check the CharLen constraints for the remaining segments only if they exist
    let kind = InstructionKind::ConsumeAllSegments(char_len);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  }

  let kind = InstructionKind::InvokeRouteHandler(ctx.route_segment_id.clone());
  ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
}
