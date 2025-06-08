use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::app::config::AppConfig;
use ::ruxy_routing::{
  Arity, RequestHandler, RouteSegment, RouteTree, SegmentEffect, UrlMatcherSequence,
};
use ruxy_routing::TrailingSlashConfig;
use ruxy_util::radix_trie::{RadixTrie, RadixTrieNode};

type Trie = RadixTrie<TokenStream>;

pub fn gen_handler_function(config: &AppConfig, routes: &RouteTree) -> TokenStream {
  let matchers = gen_matchers(config, routes);
  let global_404 = gen_global_404();

  quote! {
    async fn handler(request: internal::HyperRequest) -> internal::HandlerResult {
      let path = request.uri().path();

      #matchers
      #global_404
    }
  }
}

fn gen_matchers(config: &AppConfig, routes: &RouteTree) -> TokenStream {
  let Some(root_segment) = routes.get_root_segment() else {
    return TokenStream::new();
  };

  let trie = create_radix_trie(config, routes, root_segment);

  render_trie(&trie)

  // let debug = trie.to_flat().iter().map(|i| { let a = i.0.clone(); quote! { String::from(#a); }}).collect::<Vec<_>>();
  //
  // quote! {
  //   #(#debug)*
  // }
}

fn create_radix_trie(config: &AppConfig, routes: &RouteTree, segment: &RouteSegment) -> Trie {
  let mut self_prefix = String::new();
  let mut is_dynamic_segment = false;

  match &segment.effect {
    SegmentEffect::EmptySegment => self_prefix.push('/'),
    SegmentEffect::UrlMatcher { .. } => {
      if let Some(literal) = segment.get_literal() {
        self_prefix.push('/');
        self_prefix.push_str(literal);
      } else {
        is_dynamic_segment = true;
      }
    }
    // TODO: Custom Match segments
    _ => {}
  };

  let mut trie = RadixTrie::new();

  for child in &segment.children {
    let Some(child_segment) = routes.segments.get(child) else {
      continue;
    };

    let child_trie = create_radix_trie(config, routes, child_segment);

    trie.extend(child_trie);
  }

  if let Some(handler) = &segment.route_handler {
    let mut prefix = self_prefix.clone();

    if matches!(config.trailing_slash, TrailingSlashConfig::RequirePresent) || segment.is_root {
      prefix.push('/');
    }

    let end_of_path_cond = match (segment.is_root, &config.trailing_slash) {
      (false, TrailingSlashConfig::Ignore) => quote! { path.is_empty() || path == "/" },
      // We're handling trailing slash for "RequirePresent" as part of the prefix.
      // Root segment is also handled as part of the prefix.
      _ => quote! { path.is_empty() },
    };

    let target = gen_segment_responder(config, routes, segment, handler);
    let target = quote! { if #end_of_path_cond { #target } };

    let key = if segment.is_root { "/" } else { "" };

    trie.insert(key, target);
  }

  if is_dynamic_segment {
    // Wrap the whole trie with the dynamic segment target, which will
    // render the "wrapped" trie inside its own matching logic
    trie = create_dynamic_segment_trie(config, routes, segment, trie);
  }

  // We apply this segment's prefix to the trie here at the end, as some special targets
  // needs to wrap the unprefixed trie with its own trie (check dynamic segment handling)
  trie.with_prefix(&self_prefix)
}

/// Returns (<should prefix with slash>, TokenStream).
/// If the first value of the tuple is `true`, an additional slash should be added to the prefix.
fn create_dynamic_segment_trie(config: &AppConfig, routes: &RouteTree, segment: &RouteSegment, subtrie: Trie) -> Trie {
  let SegmentEffect::UrlMatcher { sequences } = &segment.effect else {
    unreachable!("This function only ever receives UrlMatcher-effect segments");
  };

  let mut segment_prefix: String = String::new();
  let mut segment_suffix: String = String::new();

  let mut var_name: &String = &String::new();
  let mut arity: &Arity = &Default::default();

  for (seq_index, sequence) in sequences.iter().enumerate() {
    match sequence {
      UrlMatcherSequence::Literal(literal) => {
        if seq_index == 0 {
          segment_prefix.push_str(literal);
        } else {
          segment_suffix.push_str(literal);
        }
      }
      UrlMatcherSequence::Dynamic { param_name: v, seg_count: a } => {
        var_name = v;
        arity = a;
      }
    }
  }

  let subtrie = render_trie(&subtrie);

  let path_param_value_ident = format!("path_param_{}", segment.hex);
  let path_param_value_ident = Ident::new(&path_param_value_ident, Span::mixed_site());

  let path_param_value_type = arity.get_rust_type();

  let mut prefix = String::new();

  let target = match arity {
    Arity::Exact(1) => {
      prefix.push('/');
      prefix.push_str(&segment_prefix);

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

fn render_trie(trie: &Trie) -> TokenStream {
  let items = trie.to_nodes().iter().map(|node| match node {
    RadixTrieNode::Item(item) => quote! { #item },
    RadixTrieNode::Prefix(prefix, children) => {
      let children = render_trie(children);

      quote! {
        if let Some(path) = path.strip_prefix(#prefix) {
          #children
        }
      }
    }
  });

  quote! { #(#items)* }
}

fn gen_segment_responder(
  _config: &AppConfig,
  routes: &RouteTree,
  segment: &RouteSegment,
  handler: &RequestHandler,
) -> TokenStream {
  let identifier = &segment.identifier;

  let path_params: Vec<TokenStream> = extract_idents_for_segment(segment, routes);

  quote! {
    let mut response = hyper::Response::builder();
    
    response = response.status(200);
    response = response.header("Content-Type", "text/html");
    
    let mut body = internal::ResponseBody::new();
    
    body.push(internal::Bytes::from("<!DOCTYPE html>"));
    body.push(internal::Bytes::from("<html>"));
    body.push(internal::Bytes::from("<head>"));
    body.push(internal::Bytes::from("<meta charset=\"utf-8\" />"));
    body.push(internal::Bytes::from("</head>"));
    body.push(internal::Bytes::from("<body>"));
    body.push(internal::Bytes::from("<div>Matched handler:</div>"));
    body.push(internal::Bytes::from("<div style=\"color: red;\">"));
    body.push(internal::Bytes::from(#identifier));
    body.push(internal::Bytes::from("</div>"));
    body.push(internal::Bytes::from("<div style=\"margin-top: 16px;\">Path params:</div>"));
    body.push(internal::Bytes::from("<div style=\"color: darkgreen;\">"));
    #(#path_params)*
    body.push(internal::Bytes::from("</div>"));
    body.push(internal::Bytes::from("</body>"));
    body.push(internal::Bytes::from("</html>"));
    
    return internal::HandlerResult {
      response: response.body(body)
    };
  }
}

fn extract_idents_for_segment(segment: &RouteSegment, routes: &RouteTree) -> Vec<TokenStream> {
  let mut v = Vec::new();

  if let SegmentEffect::UrlMatcher { sequences } = &segment.effect {
    let dyn_var_name = sequences.iter().find_map(|s| {
      let UrlMatcherSequence::Dynamic { param_name: var_name, .. } = s else {
        return None;
      };
      Some(var_name)
    });

    if let Some(dyn_var_name) = dyn_var_name {
      let dyn_var_ident = format!("path_param_{}", segment.hex);
      let dyn_var_ident = Ident::new(&dyn_var_ident, Span::mixed_site());

      v.push(quote! {
        body.push(internal::Bytes::from("<div>"));
        let formatted = format!("{}: {:?}", #dyn_var_name, #dyn_var_ident);
        body.push(internal::Bytes::from(formatted));
        body.push(internal::Bytes::from("</div>"));
      });
    }
  }

  if let Some(parent) = &segment.parent {
    if let Some(parent) = routes.segments.get(parent) {
      v.extend(extract_idents_for_segment(parent, routes))
    }
  }

  v
}

fn gen_global_404() -> TokenStream {
  quote! {
    let mut response = hyper::Response::builder();

    response = response.status(404);
    response = response.header("Content-Type", "text/plain");

    let mut body = internal::ResponseBody::new();

    body.push(internal::Bytes::from("Not Found"));

    internal::HandlerResult {
      response: response.body(body)
    }
  }
}

// ---------------------------------- OLD ----------------------------------

// fn _gen_segment(routes: &RouteTree, id: &str) -> TokenStream {
//   let Some(segment) = routes.segments.get(id) else {
//     return TokenStream::new();
//   };
//
//   let match_self = segment.route_handler.as_ref().map(|handler| {
//     let responder = gen_segment_responder(routes, segment, handler);
//
//     quote! {
//       // TODO: Make the trailing slash matching configurable
//       if url.is_empty() || url == "/" {
//         // Segment has been targeted for producing response
//         #responder
//       }
//     }
//   });
//
//   let match_children = gen_segment_children(segment, routes);
//
//   let inner_matchers = quote! {
//     #match_self
//     #match_children
//   };
//
//   // Wrap `inner_matchers` in URL matching conditions so that it's only executed if the segment itself matches
//   match &segment.effect {
//     SegmentEffect::Group => inner_matchers,
//     SegmentEffect::Slot { .. } => inner_matchers,
//     SegmentEffect::UrlMatcher { sequences } => {
//       gen_url_matcher_sequence_condition(segment, sequences, 0, inner_matchers)
//     }
//     SegmentEffect::EmptySegment => {
//       quote! { if url.starts_with('/') { #inner_matchers } }
//     }
//   }
// }

fn gen_url_matcher_sequence_condition(
  segment: &RouteSegment,
  sequences: &[UrlMatcherSequence],
  index: usize,
  inner: TokenStream,
) -> TokenStream {
  let sequence = &sequences[index];
  let is_last = index == sequences.len() - 1;

  let inner = if is_last { inner } else { gen_url_matcher_sequence_condition(segment, sequences, index + 1, inner) };

  match sequence {
    UrlMatcherSequence::Literal(literal) => {
      quote! { if let Some(url) = Self::strip_prefix(url, #literal) { #inner } }
    }
    UrlMatcherSequence::Dynamic { seg_count: arity, .. } => {
      let url_param_value_ident = format!("url_param_{}", segment.hex);
      let url_param_value_ident = Ident::new(&url_param_value_ident, Span::mixed_site());

      let url_param_value_type = arity.get_rust_type();

      match *arity {
        // this is the only case where we deal with prefix/suffix
        Arity::Exact(1) => {
          if is_last {
            quote! {
              if !url.is_empty() {
                let end = url.find('/').unwrap_or(url.len());
                let #url_param_value_ident: #url_param_value_type = &url[..end];
                let url = &url[end..];
                #inner
              }
            }
          } else {
            let suffix = match &sequences[index + 1] {
              UrlMatcherSequence::Literal(suffix) => suffix,
              // Only literal sequence can follow a dynamic sequence
              _ => unreachable!(),
            };

            quote! {
              let segment = &url[0..url.find('/').unwrap_or(url.len())];
              if let Some(val) = segment.strip_suffix(#suffix) {
                let #url_param_value_ident: #url_param_value_type = val;
                let url = &url[val.len()..];
                #inner
              }
            }
          }
        }
        Arity::Exact(num) => {
          assert!(num > 1);

          quote! {
            let mut #url_param_value_ident: #url_param_value_type = [const { String::new() }; #num];

            let mut rest = url;
            let mut matched = true;

            for segment_idx in 0..#num {
              if segment_idx > 0 {
                rest = rest.strip_prefix('/').unwrap_or(rest);
              }

              if rest.is_empty() {
                matched = false;
                break;
              }

              let segment = &rest[0..rest.find('/').unwrap_or(rest.len())];
              #url_param_value_ident[segment_idx] = segment;

              rest = &rest[segment.len()..];
            }

            if matched {
              let url = rest;
              #inner
            }
          }
        }
        Arity::Range(min, max) => {
          let url_param_value_initializer = match min {
            0 => quote! { Vec::new() },
            _ => quote! { ([const { String::new() }; #min], Vec::new()) },
          };

          let known_segments_loop = match min {
            0 => None,
            _ => Some(quote! {
              for segment_idx in 0..#min {
                if rest.is_empty() {
                  matched = false;
                  break;
                }

                let segment = &rest[0..rest.find('/').unwrap_or(rest.len())];
                #url_param_value_ident.0[segment_idx] = segment;

                rest = &rest[segment.len()..];
                rest = rest.strip_prefix('/').unwrap_or(rest);
              }
            }),
          };

          let unknown_segment_url_param_value_assignmnent = match min {
            0 => quote! { #url_param_value_ident.push(segment); },
            _ => quote! { #url_param_value_ident.1.push(segment); },
          };

          let unknown_segments_loop = quote! {
            if first_iteration {
              first_iteration = false;
            } else {
              rest = rest.strip_prefix('/').unwrap_or(rest);
            }

            if rest.is_empty() { break; }
            // TODO: Leaf segments ([min..x?]/something)
            //  ^ we should probably break here too ^

            let segment = &rest[0..rest.find('/').unwrap_or(rest.len())];
            #unknown_segment_url_param_value_assignmnent

            rest = &rest[segment.len()..];
          };

          let unknown_segments_loop = match max {
            Some(_) => quote! { for _ in #min..#max { #unknown_segments_loop } },
            None => quote! { loop { #unknown_segments_loop } },
          };

          quote! {
            let mut #url_param_value_ident: #url_param_value_type = #url_param_value_initializer;

            let mut rest = url;
            let mut matched = true;

            #known_segments_loop

            let mut first_iteration = true;
            #unknown_segments_loop

            if matched {
              let url = rest;
              #inner
            }
          }
        }
      }
    }
  }
}

fn gen_segment_children(segment: &RouteSegment, routes: &RouteTree) -> TokenStream {
  // let children = segment.children.iter().map(|child| {
  //   Some(gen_segment(routes, routes.segments.get(child)?))
  // });

  // TODO: Sort children by the match specificity:
  //  1. Static segments
  //  2. Dynamic segments: Exact arity (from lowest to highest)
  //  3. Dynamic segments: Range arity: Upper specified (from lowest upper to highest upper)
  //  4. Dynamic segments: Range arity: Upper unspecified (from lowest lower to highest lower)
  //  5. Group Segments

  quote! {
    let url = url.strip_prefix('/').unwrap_or(url);
    // #(#children)*
  }
}
