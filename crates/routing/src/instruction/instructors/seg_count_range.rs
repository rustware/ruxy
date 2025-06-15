use crate::instruction::create_instructions::CreateInstructionsContext;
use crate::instruction::{InstructionKind, MatchInstruction};
use crate::segment::{Arity, DynamicSequence};
use crate::sequence::{MatchDirection, RouteSequence};

pub fn instruct_seg_count_range(ctx: &mut CreateInstructionsContext, sequence: RouteSequence) {
  let RouteSequence::Dynamic(dyn_seq) = sequence else {
    unreachable!("Unexpected route sequence type");
  };

  let DynamicSequence { seg_count: Arity::Range(seg_min, seg_max), char_len, param_name, is_first, .. } = dyn_seq else {
    unreachable!("Unexpected route sequence type");
  };

  // TODO: The captured value needs to iterate differently based on the `is_first` flag.
  //  For `is_first: true` a leading slash is captured too if present (e.g. "/first/second")
  //    ^ -> "/a": ["a"], "/a/": ["a", ""], "//a": ["", "a"], "/": [""], "": []
  //  For `is_first: false`, there is no leading slash present (e.g. "first/second") 
  //    ^ -> "a": ["a"], "a/": ["a", ""], "/a": ["", "a"], "/": ["", ""], "": []
  //  This needs to be handled at runtime (when creating the iterator)
  let kind = InstructionKind::CaptureRestOfPath(param_name);
  ctx.instructions.push(MatchInstruction { kind, ..Default::default() });

  if seg_min > 0 {
    let kind = InstructionKind::ConsumeSegmentCount(seg_min, char_len, MatchDirection::Ltr);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  } else if is_first {
    // Handle the leading slash if the 0..? sequence is the first in route segment
    let kind = InstructionKind::PathEmptyOrConsumeSlash;
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  }
  
  if let Some(max) = seg_max.map(|max| max - seg_min) {
    let kind = InstructionKind::ConsumeUpToSegmentCount(max, char_len);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
    let kind = InstructionKind::CheckEndOfUrl;
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  } else if !matches!(char_len, Arity::Range(0, None)) {
    // Check the CharLen constraints for the remaining segments only if they exist
    let kind = InstructionKind::ConsumeAllSegments(char_len);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  }

  let kind = InstructionKind::InvokeRouteHandler(ctx.route_segment_id.clone());
  ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
}
