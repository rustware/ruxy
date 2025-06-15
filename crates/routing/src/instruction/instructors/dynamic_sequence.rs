use crate::instruction::create_instructions::CreateInstructionsContext;
use crate::instruction::{InstructionKind, MatchInstruction};
use crate::segment::{Arity, DynamicSequence};
use crate::sequence::{MatchDirection, RouteSequence};

pub fn instruct_dynamic_sequence(ctx: &mut CreateInstructionsContext, sequence: DynamicSequence) {
  let Arity::Exact(seg_count) = sequence.seg_count else {
    unreachable!("SegCount:Range sequences are handled upstream");
  };

  if seg_count == 0 {
    // SegCount:0 sequences are ignored as they don't match anything.
    return;
  }

  let direction = if ctx.path_rtl { MatchDirection::Rtl } else { MatchDirection::Ltr };

  if let Arity::Exact(char_count) = sequence.char_len {
    let char_count = char_count * seg_count + seg_count - 1;
    let kind = InstructionKind::CaptureExactChars(sequence.param_name, char_count, direction);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });

    let kind = InstructionKind::ConsumeSegmentCount(seg_count, sequence.char_len, direction);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
    return;
  }

  if let Arity::Range(min, max) = sequence.char_len {
    let (sequences, char_offset) = find_segment_boundary(ctx);

    let kind = InstructionKind::ConsumeIntoView(direction, char_offset);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });

    for sequence in sequences {
      match sequence {
        RouteSequence::Literal(literal) => {
          let direction = if ctx.path_rtl { MatchDirection::Ltr } else { MatchDirection::Rtl };
          let kind = InstructionKind::ConsumeLiteralInView(literal, direction);
          ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
        }
        RouteSequence::Dynamic(DynamicSequence {
          char_len: Arity::Exact(char_count),
          seg_count: Arity::Exact(1),
          param_name,
          ..
        }) => {
          let direction = if ctx.path_rtl { MatchDirection::Ltr } else { MatchDirection::Rtl };
          
          let kind = InstructionKind::CaptureExactCharsInView(param_name, char_count, direction);
          ctx.instructions.push(MatchInstruction { kind, ..Default::default() });

          let kind = InstructionKind::ConsumeExactCharsInView(char_count, direction);
          ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
        },
        _ => unreachable!("Unexpected sequence contained in view"),
      }
    }

    let kind = InstructionKind::CheckCharLenInRestOfView(min, max);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });

    let kind = InstructionKind::CaptureRestOfView(sequence.param_name);
    ctx.instructions.push(MatchInstruction { kind, ..Default::default() });
  }
}

/// Returns:
/// 1. all sequences in between the current sequence and the boundary sequence
/// 2. a character offset to exclude from the view on its end (or start if RTL)
fn find_segment_boundary(ctx: &mut CreateInstructionsContext) -> (Vec<RouteSequence>, usize) {
  let mut char_count_offset = 0;

  let mut finder = |_: _, seq: &RouteSequence| match seq {
    RouteSequence::Slash => true,
    RouteSequence::Dynamic(DynamicSequence {
      seg_count: Arity::Exact(2..) | Arity::Range(2.., _),
      char_len: Arity::Exact(char_count),
      ..
    }) => {
      char_count_offset = *char_count;
      true
    }
    _ => false,
  };

  let mut iter = ctx.sequences.iter().enumerate();

  let found = if ctx.path_rtl { iter.rfind(|(i, seq)| finder(*i, seq)) } else { iter.find(|(i, seq)| finder(*i, seq)) };

  let Some((boundary_seq_index, _)) = found else {
    return (ctx.sequences.drain(..).collect(), 0);
  };

  let mut slice: Vec<RouteSequence> = if ctx.path_rtl {
    ctx.sequences.drain(boundary_seq_index + 1..).collect()
  } else {
    ctx.sequences.drain(..boundary_seq_index).collect()
  };

  if !ctx.path_rtl {
    slice.reverse();
  }

  (slice, char_count_offset)
}

// TODO: Document the concept of RangeChar breakers
// There is a rule. Two RangeChars cannot be in the same route segment,
// UNLESS there is a RangeChar breaker between them.
// Valid RangeChar breakers:
// - [2](n), [2..](n)
