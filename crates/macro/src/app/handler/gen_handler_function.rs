use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::app::config::AppConfig;
use ::ruxy_routing::{
  DynamicSequenceArity, RequestHandler, RouteSegment, RouteTree, SegmentEffect, UrlMatcherSequence,
};
use ruxy_routing::TrailingSlashConfig;
use ruxy_util::RadixTrie;

pub fn gen_handler_function(_config: &AppConfig, routes: &RouteTree) -> TokenStream {
  let root_segment = match routes.get_root_segment() {
    Some(root) => gen_segment(routes, root),
    None => TokenStream::new(),
  };

  let global_404 = gen_global_404();

  quote! {
    async fn handler(request: internal::HyperRequest) -> internal::HandlerResult {
      let url = request.uri().path();

      #root_segment
      #global_404
    }
  }
}

fn gen_segment(routes: &RouteTree, segment: &RouteSegment) -> TokenStream {
  let match_self = segment.route_handler.as_ref().map(|handler| {
    let responder = gen_segment_responder(routes, segment, handler);

    quote! {
      if url.is_empty() || url == "/" {
        // Segment has been targeted for producing response
        #responder
      }
    }
  });

  let children = get_flat_children(routes, segment);

  let mut empty_url_segments: Vec<TokenStream> = Vec::new();
  let mut static_prefix_segments: Vec<(&str, &RouteSegment)> = Vec::new();
  let mut dynamic_optional_segments: Vec<&RouteSegment> = Vec::new();
  let mut dynamic_required_segments: Vec<&RouteSegment> = Vec::new();
  let mut slots: Vec<&RouteSegment> = Vec::new();

  for child in children {
    match &child.effect {
      SegmentEffect::UrlMatcher { sequences } => {
        if let Some(UrlMatcherSequence::Literal(literal)) = sequences.first() {
          static_prefix_segments.push((literal, child));
        } else if child.effect.is_optional() {
          dynamic_optional_segments.push(child);
        } else {
          dynamic_required_segments.push(child);
        }
      }
      SegmentEffect::Slot { .. } => {
        slots.push(child);
      }
      SegmentEffect::EmptySegment => {
        empty_url_segments.push(gen_segment(routes, child));
      }
      SegmentEffect::Group => {
        // No-op. Groups are already recursively flattened.
      }
    };
  }

  let static_prefix_matchers = gen_static_prefix_matcher(routes, static_prefix_segments);
  
  let dynamic_optional_segment_matchers =
    dynamic_optional_segments.iter().map(|segment| gen_dynamic_segment_matcher(segment));
  let dynamic_optional_segment_matchers = quote! { #(#dynamic_optional_segment_matchers)* };
  
  let dynamic_required_segment_matchers =
    dynamic_required_segments.iter().map(|segment| gen_dynamic_segment_matcher(segment));
  let dynamic_required_segment_matchers = quote! { #(#dynamic_required_segment_matchers)* };

  // TODO: Sort children by the match specificity:
  //  1. Static segments
  //  2. Dynamic segments: Exact arity (from lowest to highest)
  //  3. Dynamic segments: Range arity: Upper specified (from lowest upper to highest upper)
  //  4. Dynamic segments: Range arity: Upper unspecified (from lowest lower to highest lower)
  //  5. Group Segments

  let empty_url_segments = gen_empty_url_segments(empty_url_segments);

  let mut separated_segment_matchers = quote! {
    #empty_url_segments
    #static_prefix_matchers
    #dynamic_required_segment_matchers
    #dynamic_optional_segment_matchers
  };
  
  if !separated_segment_matchers.is_empty() {
    separated_segment_matchers = quote! {
      if let Some(url) = url.strip_prefix('/') {
        #separated_segment_matchers
      }
    };
  }
  
  quote! {
    #match_self

    #separated_segment_matchers
  }
}

fn gen_static_prefix_matcher(routes: &RouteTree, segments: Vec<(&str, &RouteSegment)>) -> TokenStream {
  let trie = RadixTrie::build(segments);
  let children = gen_static_prefix_matcher_node(routes, &trie);

  quote! { #children }
}

fn gen_static_prefix_matcher_node(routes: &RouteTree, children: &Vec<RadixTrie<&RouteSegment>>) -> TokenStream {
  let matchers = children.iter().map(|node| {
    match node {
      RadixTrie::Prefix(prefix, children) => {
        let children = gen_static_prefix_matcher_node(routes, children);

        quote! {
          if let Some(url) = Self::strip_prefix_decode(url, #prefix) {
            #children
          }
        }
      }
      RadixTrie::Item(segment) => {
        if let SegmentEffect::UrlMatcher { sequences, .. } = &segment.effect {
          if sequences.len() == 1 {
            // Skip dynamic matching, this segment's UrlMatcher has only a single literal sequence
            return gen_segment(routes, segment);
          }
        }

        gen_dynamic_segment_matcher(segment)
      }
    }
  });

  quote! { #(#matchers)* }
}

fn gen_dynamic_segment_matcher(segment: &RouteSegment) -> TokenStream {
  // There is no static prefix, that's already handled by the static prefix matcher
  // At the end, call `gen_next_segment(segment)` to move to the next level
  quote! { todo!("dynamic segment matcher here") }
}

fn gen_empty_url_segments(segments: Vec<TokenStream>) -> TokenStream {
  if segments.is_empty() { return TokenStream::new(); }
  
  quote! {
    if url.is_empty() || url.starts_with('/') {
      let url = url.strip_prefix('/').unwrap_or(url);
      #(#segments)*
    }
  }
}

// Flattens children from Route Groups recursively
fn get_flat_children<'a>(routes: &'a RouteTree, segment: &RouteSegment) -> Vec<&'a RouteSegment> {
  let mut children = Vec::new();

  for child in &segment.children {
    let Some(child_segment) = routes.segments.get(child) else { continue };

    if let SegmentEffect::Group = child_segment.effect {
      children.extend(get_flat_children(routes, child_segment));
      continue;
    }

    children.push(child_segment);
  }

  children
}

fn _gen_segment(routes: &RouteTree, id: &str) -> TokenStream {
  let Some(segment) = routes.segments.get(id) else {
    return TokenStream::new();
  };

  let match_self = segment.route_handler.as_ref().map(|handler| {
    let responder = gen_segment_responder(routes, segment, handler);

    quote! {
      // TODO: Make the trailing slash matching configurable
      if url.is_empty() || url == "/" {
        // Segment has been targeted for producing response
        #responder
      }
    }
  });

  let match_children = gen_segment_children(segment, routes);

  let inner_matchers = quote! {
    #match_self
    #match_children
  };

  // Wrap `inner_matchers` in URL matching conditions so that it's only executed if the segment itself matches
  match &segment.effect {
    SegmentEffect::Group => inner_matchers,
    SegmentEffect::Slot { .. } => inner_matchers,
    SegmentEffect::UrlMatcher { sequences } => {
      gen_url_matcher_sequence_condition(segment, sequences, 0, inner_matchers)
    }
    SegmentEffect::EmptySegment => {
      quote! { if url.starts_with('/') { #inner_matchers } }
    }
  }
}

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
      quote! { if let Some(url) = Self::strip_prefix_decode(url, #literal) { #inner } }
    }
    UrlMatcherSequence::Dynamic { arity, .. } => {
      let url_param_value_ident = format!("url_param_{}", segment.hex);
      let url_param_value_ident = Ident::new(&url_param_value_ident, Span::mixed_site());

      let url_param_value_type = arity.get_rust_type();

      match *arity {
        // this is the only case where we deal with prefix/suffix
        DynamicSequenceArity::Exact(1) => {
          if is_last {
            quote! {
              if !url.is_empty() {
                let end = url.find('/').unwrap_or(url.len());
                let #url_param_value_ident: #url_param_value_type = Self::decode_dyn_segment_value(&url[..end]);
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
                let #url_param_value_ident: #url_param_value_type = Self::decode_dyn_segment_value(val);
                let url = &url[val.len()..];
                #inner
              }
            }
          }
        }
        DynamicSequenceArity::Exact(num) => {
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
              #url_param_value_ident[segment_idx] = Self::decode_dyn_segment_value(segment);

              rest = &rest[segment.len()..];
            }

            if matched {
              let url = rest;
              #inner
            }
          }
        }
        DynamicSequenceArity::Range(min, max) => {
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
                #url_param_value_ident.0[segment_idx] = Self::decode_dyn_segment_value(segment);

                rest = &rest[segment.len()..];
                rest = rest.strip_prefix('/').unwrap_or(rest);
              }
            }),
          };

          let unknown_segment_url_param_value_assignmnent = match min {
            0 => quote! { #url_param_value_ident.push(Self::decode_dyn_segment_value(segment)); },
            _ => quote! { #url_param_value_ident.1.push(Self::decode_dyn_segment_value(segment)); },
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

fn gen_segment_responder(routes: &RouteTree, segment: &RouteSegment, handler: &RequestHandler) -> TokenStream {
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
      let UrlMatcherSequence::Dynamic { var_name, .. } = s else {
        return None;
      };
      Some(var_name)
    });

    if let Some(dyn_var_name) = dyn_var_name {
      let dyn_var_ident = format!("url_param_{}", segment.hex);
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
