use crate::segment::{Arity, DynamicSequence, RouteSegment, SegmentEffect, TypedSequence};
use crate::sequence::RouteSequence;

pub fn get_segment_sequences(segment: &RouteSegment) -> Vec<RouteSequence> {
  match &segment.effect {
    SegmentEffect::EmptySegment => {
      vec![RouteSequence::Slash]
    }
    SegmentEffect::CustomMatch { .. } => {
      vec![RouteSequence::Custom(segment.identifier.clone())]
    }
    SegmentEffect::UrlMatcher { sequences: url_matcher_sequences } => {
      let mut sequences = vec![];

      let matcher_sequences = url_matcher_sequences.iter().filter(|seq| {
        // Filter out SegCount:Exact(0) sequences up front
        !matches!(seq.typed, TypedSequence::Dynamic(DynamicSequence { seg_count: Arity::Exact(0), .. }))
      });
      
      let matcher_sequences = matcher_sequences.collect::<Vec<_>>();
      
      for (index, seq) in matcher_sequences.iter().enumerate() {
        match &seq.typed {
          TypedSequence::Literal(literal) => {
            if index == 0 {
              sequences.push(RouteSequence::Slash);
            }

            sequences.push(RouteSequence::Literal(literal.clone()));
          }
          TypedSequence::Dynamic(seq) => {
            if index == 0 && (seq.seg_count.get_min() > 0 || matcher_sequences.len() > 1) {
              sequences.push(RouteSequence::Slash);
            }
            
            sequences.push(RouteSequence::Dynamic(seq.clone()));
          }
        }
      }

      sequences
    }
    _ => vec![],
  }
}
