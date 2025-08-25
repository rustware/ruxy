use crate::routing::instruction::create_instructions::CreateInstructionsContext;
use crate::routing::instruction::{MatchInstructionKind, MatchDirection, MatchInstruction};
use crate::routing::segment::{Arity, DynamicSequence};
use crate::routing::sequence::RouteSequence;

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
  let kind = MatchInstructionKind::CaptureRestOfPath(param_name);
  ctx.instructions.push(MatchInstruction { kind, ..Default::default() });

  if seg_min > 0 {
    let kind = MatchInstructionKind::ConsumeSegmentCount(seg_min, char_len, MatchDirection::Ltr);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  }

  if let Some(max) = seg_max.map(|max| max - seg_min) {
    let kind = MatchInstructionKind::ConsumeUpToSegmentCount(max, char_len);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
    let kind = MatchInstructionKind::CheckEndOfPath;
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  } else if !matches!(char_len, Arity::Range(0, None)) {
    // Check the CharLen constraints for the remaining segments only if they exist
    let kind = MatchInstructionKind::ConsumeAllSegments(char_len);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  }

  let kind = MatchInstructionKind::InvokeRouteHandler(ctx.route_segment_id.clone());
  ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
}
