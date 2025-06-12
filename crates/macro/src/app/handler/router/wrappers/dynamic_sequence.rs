use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_routing::{Arity, UrlMatcherSequence, TypedSequence, DynamicSequence, RouteSequence, RouteSequenceMatcher};
use ::ruxy_util::radix_trie::RadixTrie;

use crate::app::handler::router::context::GenContext;
use crate::app::handler::router::render::render_trie;

type Trie = RadixTrie<TokenStream>;

pub fn with_dynamic_sequence(ctx: &GenContext, sequence: &RouteSequence, subtrie: Trie) -> Trie {
  let segment = &ctx.routes.segments[&sequence.containing_segment_id];
  
  let RouteSequenceMatcher::Dynamic(dyn_seq) = &sequence.matcher else {
    unreachable!("Unexpected sequence matcher");
  };
  
  let subtrie = render_trie(&subtrie, sequence.direction);

  let path_param_value_ident = format!("path_param_{}", segment.hex);
  let path_param_value_ident = Ident::new(&path_param_value_ident, Span::mixed_site());
  
  let path_param_value_type = dyn_seq.get_rust_type();

  let mut prefix = String::new();
  
  let target = match dyn_seq.seg_count {
    Arity::Exact(1) => {
      if segment_suffix.is_empty() {
        quote! {
          let (value, path) = Self::split_segment_end(path);
          let #path_param_value_ident: #path_param_value_type = value.into();
          #subtrie
        }
      } else {
        quote! {
          if let Some((value, path)) = Self::strip_segment_suffix(path, #segment_suffix) {
            let #path_param_value_ident: #path_param_value_type = value.into();
            #subtrie
          }
        }
      }
    }
    Arity::Exact(count) => {
      // "Exact(0)" arities are converted to Groups
      // "Exact(1)" is handled above
      assert!(*count > 1);

      prefix.push('/');

      quote! {
        let mut #path_param_value_ident: #path_param_value_type = [const { String::new() }; #count];

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
