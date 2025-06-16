use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ::ruxy_routing::instruction::{InstructionKind, MatchDirection};
use ::ruxy_routing::segment::Arity;

use crate::app::handler::router::context::GenContext;
use crate::app::handler::router::gen_segment_responder;

pub fn render_instruction(ctx: &GenContext, kind: &InstructionKind, children: TokenStream) -> TokenStream {
  match kind {
    InstructionKind::Skip => children,
    InstructionKind::ConsumeIntoView(MatchDirection::Ltr, 0) => quote! {
      let (view, path) = path.find('/').map(|i| path.split_at(i)).unwrap_or((path, ""));
      #children
    },
    InstructionKind::ConsumeIntoView(MatchDirection::Ltr, offset) => quote! {
      if let Some((view, path)) = path.find('/').and_then(|i| i.checked_sub(#offset)).and_then(|i| path.split_at_checked(i)) {
        #children
      };
    },
    InstructionKind::ConsumeIntoView(MatchDirection::Rtl, 0) => quote! {
      let (path, view) = path.rfind('/').map(|i| path.split_at(i + 1)).unwrap_or(("", path));
      #children
    },
    InstructionKind::ConsumeIntoView(MatchDirection::Rtl, offset) => {
      let offset = offset + 1;
      quote! {
        if let Some((path, view)) = path.rfind('/').and_then(|i| path.split_at_checked(i + #offset)) { #children };
      }
    }
    InstructionKind::CaptureRestOfPath(param_name) => {
      let ident = create_param_ident(param_name);
      quote! { let #ident = path; #children }
    }
    // Special-case the {_[1](n)} (LTR)
    InstructionKind::ConsumeSegmentCount(1, Arity::Exact(char_count), MatchDirection::Ltr) => quote! {
      if let Some((_, path)) = path.split_at_checked(#char_count) { #children }
    },
    // Special-case the {_[1](n)} (RTL)
    InstructionKind::ConsumeSegmentCount(1, Arity::Exact(char_count), MatchDirection::Rtl) => quote! {
      if let Some((path, _)) = path.len().checked_sub(#char_count).and_then(|i| path.split_at_checked(i)) { #children }
    },
    InstructionKind::ConsumeSegmentCount(count, char_len, direction) => {
      let strip_method = match direction {
        MatchDirection::Ltr => quote! { strip_prefix },
        MatchDirection::Rtl => quote! { strip_suffix },
      };

      let split_segment_end = match direction {
        MatchDirection::Ltr => {
          quote! { let (segment, stripped) = rest.find('/').map(|i| rest.split_at(i)).unwrap_or((rest, "")); }
        }
        MatchDirection::Rtl => {
          quote! { let (stripped, segment) = rest.rfind('/').map(|i| rest.split_at(i + 1)).unwrap_or(("", rest)); }
        }
      };

      let check_char_len = gen_char_len_check_for_segment(char_len);

      quote! {
        let mut rest = path;
        let mut matched = true;

        for i in 0..#count {
          if i != 0 {
            let Some(stripped) = rest.#strip_method('/') else {
              matched = false;
              break;
            };

            rest = stripped;
          }

          #split_segment_end
          rest = stripped;

          #check_char_len
        }

        if matched { let path = rest; #children }
      }
    }
    InstructionKind::ConsumeUpToSegmentCount(count, char_len) => {
      let check_char_len = gen_char_len_check_for_segment(char_len);

      quote! {
        let mut rest = path;
        let mut matched = true;

        for _ in 0..#count {
          let Some(stripped) = rest.strip_prefix('/') else {
            break;
          };

          let (segment, stripped) = stripped.find('/').map(|i| stripped.split_at(i)).unwrap_or((stripped, ""));
          #check_char_len
          rest = stripped;
        }

        if matched { let path = rest; #children }
      }
    }
    InstructionKind::ConsumeAllSegments(char_len) => {
      let check_char_len = gen_char_len_check_for_segment(char_len);

      quote! {
        let mut rest = path;
        let mut matched = true;

        while let Some(stripped) = rest.strip_prefix('/') {
          let (segment, stripped) = stripped.find('/').map(|i| stripped.split_at(i)).unwrap_or((stripped, ""));
          #check_char_len
          rest = stripped;
        }

        if matched { #children }
      }
    }
    InstructionKind::PathEmptyOrConsumeSlash => quote! {
      if let Some(path) = path.strip_prefix('/').or_else(|| if path.is_empty() { Some("") } else { None }) { #children }
    },
    InstructionKind::CaptureExactChars(param_name, count, MatchDirection::Ltr) => {
      let ident = create_param_ident(param_name);
      quote! { if let Some((#ident, _)) = path.split_at_checked(#count) { #children } }
    }
    InstructionKind::CaptureExactChars(param_name, count, MatchDirection::Rtl) => {
      let ident = create_param_ident(param_name);
      quote! { if let Some((_, #ident)) = path.len().checked_sub(#count).and_then(|i| path.split_at_checked(i)) { #children } }
    }
    InstructionKind::CaptureExactCharsInView(param_name, count, MatchDirection::Ltr) => {
      let ident = create_param_ident(param_name);
      quote! { if let Some((#ident, _)) = view.split_at_checked(#count) { #children } }
    }
    InstructionKind::CaptureExactCharsInView(param_name, count, MatchDirection::Rtl) => {
      let ident = create_param_ident(param_name);
      quote! { if let Some((_, #ident)) = view.len().checked_sub(#count).and_then(|i| view.split_at_checked(i)) { #children } }
    }
    InstructionKind::ConsumeExactCharsInView(count, MatchDirection::Ltr) => quote! {
      if let Some((_, view)) = view.split_at_checked(#count) { #children }
    },
    InstructionKind::ConsumeExactCharsInView(count, MatchDirection::Rtl) => quote! {
      if let Some((view, _)) = view.len().checked_sub(#count).and_then(|i| view.split_at_checked(i)) { #children }
    },
    InstructionKind::CaptureRestOfView(param_name) => {
      let ident = create_param_ident(param_name);
      quote! { let #ident = view; #children }
    }
    InstructionKind::CheckCharLenInRestOfView(0, None) => children,
    InstructionKind::CheckCharLenInRestOfView(0, Some(max)) => quote! {
      if view.len() <= #max { #children }
    },
    InstructionKind::CheckCharLenInRestOfView(min, None) => quote! {
      if view.len() >= #min { #children }
    },
    InstructionKind::CheckCharLenInRestOfView(min, Some(max)) => quote! {
      if view.len() >= #min && view.len() <= #max { #children }
    },
    InstructionKind::InvokeCustomMatcher(segment_id) => quote! {
      // todo
    },
    InstructionKind::ConsumeLiteral(literal, MatchDirection::Ltr) => quote! {
      if let Some(path) = path.strip_prefix(#literal) { #children }
    },
    InstructionKind::ConsumeLiteral(literal, MatchDirection::Rtl) => quote! {
      if let Some(path) = path.strip_suffix(#literal) { #children }
    },
    InstructionKind::ConsumeLiteralInView(literal, MatchDirection::Ltr) => quote! {
      if let Some(view) = view.strip_prefix(#literal) { #children }
    },
    InstructionKind::ConsumeLiteralInView(literal, MatchDirection::Rtl) => quote! {
      if let Some(view) = view.strip_suffix(#literal) { #children }
    },
    InstructionKind::CheckEndOfPath => quote! {
      if path.is_empty() { #children }
    },
    InstructionKind::InvokeRouteHandler(segment_id) => {
      let segment = &ctx.routes.segments[segment_id];
      gen_segment_responder(ctx, segment)
    }
    InstructionKind::InvokeNotFoundHandler(segment_id) => quote! {
      // todo
    },
  }
}

fn create_param_ident(param_name: &str) -> Ident {
  Ident::new(&format!("path_param_{param_name}"), Span::mixed_site())
}

/// Generates a code that checks the length of remaining characters in the segment.
/// The `segment` variable must be already generated, as well as the `matched` variable,
/// which will be set to `false` if the segment doesn't match. This can only be placed
/// inside a loop.
fn gen_char_len_check_for_segment(char_len: &Arity) -> TokenStream {
  match char_len {
    Arity::Exact(len) => quote! { if segment.len() != #len { matched = false; break; } },
    Arity::Range(0, None) => TokenStream::new(),
    Arity::Range(min, None) => quote! { if segment.len() < #min { matched = false; break; } },
    Arity::Range(min, Some(max)) => {
      quote! { if segment.len() < #min || segment.len() > #max { matched = false; break; } }
    }
  }
}
