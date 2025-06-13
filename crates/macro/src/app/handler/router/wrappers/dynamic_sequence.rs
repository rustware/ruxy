use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_routing::{Arity, MatchDirection, RouteSequence, RouteSequenceMatcher};
use ::ruxy_util::radix_trie::RadixTrie;

use crate::app::handler::router::context::GenContext;
use crate::app::handler::router::render::render_trie;

type Trie = RadixTrie<TokenStream>;

pub fn with_dynamic_sequence(ctx: &GenContext, sequence: &RouteSequence, subtrie: Trie) -> Trie {
  let segment = &ctx.routes.segments[&sequence.containing_segment_id];
  let concludes_segment = sequence.concludes_segment_id.as_ref().map(|id| {
    &ctx.routes.segments[&sequence.containing_segment_id]
  });

  let RouteSequenceMatcher::Dynamic(seq) = &sequence.matcher else {
    unreachable!("Unexpected sequence matcher type");
  };

  let subtrie = render_trie(&subtrie, sequence.url_segment_direction);

  let path_param_value_ident = format!("path_param_{}", seq.param_name);
  let path_param_value_ident = Ident::new(&path_param_value_ident, Span::mixed_site());

  let mut prefix = String::new();

  let target = match (seq.seg_count, seq.char_len, sequence.url_path_direction, sequence.url_segment_direction) {
    (Arity::Exact(1), Arity::Exact(char_len), MatchDirection::Ltr, MatchDirection::Ltr) => {
      quote! {
        if let Some((#path_param_value_ident, path)) = path.split_at_checked(#char_len) {
          if !#path_param_value_ident.contains('/') {
            #subtrie
          }
        };
      }
    }
    (Arity::Exact(1), Arity::Exact(char_len), MatchDirection::Ltr, MatchDirection::Rtl) => {
      quote! {
        let (path, segment) = Self::split_segment_end(path);
        
        if let Some((#path_param_value_ident, path)) = path.split_at_checked(#char_len) {
          if !#path_param_value_ident.contains('/') {
            #subtrie
          }
        };
      }
    }
    Arity::Exact(count) => {
      // "Exact(0)" arities are converted to Groups
      // "Exact(1)" is handled above
      assert!(count > 1);

      prefix.push('/');

      quote! {
        let mut #path_param_value_ident: &str = [const { String::new() }; #count];

        let mut path = path;
        let mut matched = true;

        for segment_idx in 0..#count {
          // First segment is always matched (slash is part of prefix)
          if segment_idx != 0 {
            let Some(rest) = path.strip_prefix('/') else {
              // No next URL segment to match this route segment
              matched = false;
              break;
            };

            path = rest;
          }

          let (value, rest) = Self::split_segment_end(path);
          #path_param_value_ident[segment_idx] = value.into();
          path = rest;
        }

        if matched { #subtrie }
      }
    }
    Arity::Range(min, max) => {
      let mut required_segments_loop = TokenStream::new();

      if *min > 0 {
        prefix.push('/');

        required_segments_loop = quote! {
          for segment_idx in 0..#min {
            // First segment is always matched (slash is part of prefix)
            if segment_idx != 0 {
              let Some(rest) = path.strip_prefix('/') else {
                // No next URL segment to match this route segment
                matched = false;
                break;
              };

              path = rest;
            }

            let (value, rest) = Self::split_segment_end(path);
            #path_param_value_ident.0[segment_idx] = value.into();
            path = rest;
          }
        };
      }

      let path_param_value_initializer = match min {
        0 => quote! { Vec::new() },
        _ => quote! { ([const { String::new() }; #min], Vec::new()) },
      };

      let path_param_value_assignment = match min {
        0 => quote! { #path_param_value_ident.push(value.into()); },
        _ => quote! { #path_param_value_ident.1.push(value.into()); },
      };

      let optional_segments_loop_definition = match max {
        Some(max) => quote! { for _ in 0..#max },
        None => quote! { loop },
      };

      quote! {
        let mut #path_param_value_ident: #path_param_value_type = #path_param_value_initializer;

        let mut path = path;
        let mut matched = true;

        #required_segments_loop

        if matched {
          // Extract values from the optional segments
          #optional_segments_loop_definition {
            let Some(rest) = path.strip_prefix('/') else {
              // No more segments
              break;
            };

            let (value, rest) = Self::split_segment_end(rest);
            #path_param_value_assignment
            path = rest;
          }

          #subtrie
        }
      }
    }
  };

  RadixTrie::from([(prefix, target)])
}
