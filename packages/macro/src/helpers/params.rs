use ::ruxy_core::routing::segment::{parse_segment, DynamicSequence, SegmentEffect, TypedSequence, UrlMatcherSequence};

pub fn get_params_for_segment_id(segment_id: &str) -> Vec<DynamicSequence> {
  let mut params = vec![];
  
  // Separators in Segment IDs are standardized to '/' for all platforms.
  let dir_names = segment_id.split('/');
  
  for dir_name in dir_names {
    let Ok(SegmentEffect::UrlMatcher { sequences }) = parse_segment(dir_name) else {
      continue;
    };
    
    let dyn_seqs = sequences.into_iter().filter_map(|seq| {
      if let UrlMatcherSequence { typed: TypedSequence::Dynamic(dyn_seq), .. } = seq {
        Some(dyn_seq)
      } else {
        None
      }
    });
    
    params.extend(dyn_seqs);
  }

  params
}