use crate::routing::segment::{RouteSegment, SegmentEffect, TypedSequence};
use crate::routing::sequence::RouteSequence;

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

      for (index, seq) in url_matcher_sequences.iter().enumerate() {
        match &seq.typed {
          TypedSequence::Literal(literal) => {
            if index == 0 {
              sequences.push(RouteSequence::Slash);
            }

            sequences.push(RouteSequence::Literal(literal.clone()));
          }
          TypedSequence::Dynamic(seq) => {
            if index == 0 && seq.seg_count.get_min() > 0 {
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
