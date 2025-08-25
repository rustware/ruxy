use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::routing::instruction::{MatchInstructionKind, MatchDirection};
use crate::routing::segment::Arity;

use crate::build::app::context::GenContext;
use crate::build::app::handler::responder::gen_segment_responder;

pub fn render_instruction(ctx: &GenContext, kind: &MatchInstructionKind, children: TokenStream) -> TokenStream {
  match kind {
    MatchInstructionKind::Skip => children,
    MatchInstructionKind::ConsumeIntoView(MatchDirection::Ltr, 0) => quote! {
      let (view, path) = path.find('/').map(|i| path.split_at(i)).unwrap_or((path, ""));
      #children
    },
    MatchInstructionKind::ConsumeIntoView(MatchDirection::Ltr, offset) => quote! {
      if let Some((view, path)) = path.find('/').and_then(|i| i.checked_sub(#offset)).and_then(|i| path.split_at(i)) {
        #children
      };
    },
    MatchInstructionKind::ConsumeIntoView(MatchDirection::Rtl, 0) => quote! {
      let (path, view) = path.rfind('/').map(|i| path.split_at(i + 1)).unwrap_or(("", path));
      #children
    },
    MatchInstructionKind::ConsumeIntoView(MatchDirection::Rtl, offset) => {
      let offset = offset + 1;
      quote! {
        if let Some((path, view)) = path.rfind('/').and_then(|i| path.split_at_checked(i + #offset)) { #children };
      }
    }
    MatchInstructionKind::CaptureRestOfPath(param_name) => {
      let ident = create_param_ident(param_name);
      quote! { let #ident = path; #children }
    }
    // Special-case the {_[1](n)} (LTR)
    MatchInstructionKind::ConsumeSegmentCount(1, Arity::Exact(char_count), MatchDirection::Ltr) => quote! {
      if let Some((_, path)) = path.split_at_checked(#char_count) { #children }
    },
    // Special-case the {_[1](n)} (RTL)
    MatchInstructionKind::ConsumeSegmentCount(1, Arity::Exact(char_count), MatchDirection::Rtl) => quote! {
      if let Some((path, _)) = path.len().checked_sub(#char_count).and_then(|i| path.split_at_checked(i)) { #children }
    },
    MatchInstructionKind::ConsumeSegmentCount(count, char_len, direction) => {
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
    MatchInstructionKind::ConsumeUpToSegmentCount(count, char_len) => {
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
    MatchInstructionKind::ConsumeAllSegments(char_len) => {
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
    MatchInstructionKind::PathEmptyOrConsumeSlash => quote! {
      if let Some(path) = path.strip_prefix('/').or_else(|| if path.is_empty() { Some("") } else { None }) { #children }
    },
    MatchInstructionKind::CaptureExactChars(param_name, count, MatchDirection::Ltr) => {
      let ident = create_param_ident(param_name);
      quote! { if let Some((#ident, _)) = path.split_at_checked(#count) { #children } }
    }
    MatchInstructionKind::CaptureExactChars(param_name, count, MatchDirection::Rtl) => {
      let ident = create_param_ident(param_name);
      quote! { if let Some((_, #ident)) = path.len().checked_sub(#count).and_then(|i| path.split_at_checked(i)) { #children } }
    }
    MatchInstructionKind::CaptureExactCharsInView(param_name, count, MatchDirection::Ltr) => {
      let ident = create_param_ident(param_name);
      quote! { if let Some((#ident, _)) = view.split_at_checked(#count) { #children } }
    }
    MatchInstructionKind::CaptureExactCharsInView(param_name, count, MatchDirection::Rtl) => {
      let ident = create_param_ident(param_name);
      quote! { if let Some((_, #ident)) = view.len().checked_sub(#count).and_then(|i| view.split_at_checked(i)) { #children } }
    }
    MatchInstructionKind::ConsumeExactCharsInView(count, MatchDirection::Ltr) => quote! {
      if let Some((_, view)) = view.split_at_checked(#count) { #children }
    },
    MatchInstructionKind::ConsumeExactCharsInView(count, MatchDirection::Rtl) => quote! {
      if let Some((view, _)) = view.len().checked_sub(#count).and_then(|i| view.split_at_checked(i)) { #children }
    },
    MatchInstructionKind::CaptureRestOfView(param_name) => {
      let ident = create_param_ident(param_name);
      quote! { let #ident = view; #children }
    }
    MatchInstructionKind::CheckCharLenInRestOfView(0, None) => children,
    MatchInstructionKind::CheckCharLenInRestOfView(0, Some(max)) => quote! {
      if view.len() <= #max { #children }
    },
    MatchInstructionKind::CheckCharLenInRestOfView(min, None) => quote! {
      if view.len() >= #min { #children }
    },
    MatchInstructionKind::CheckCharLenInRestOfView(min, Some(max)) => quote! {
      if view.len() >= #min && view.len() <= #max { #children }
    },
    MatchInstructionKind::InvokeCustomMatcher(segment_id) => quote! {
      // todo
    },
    MatchInstructionKind::ConsumeLiteral(literal, MatchDirection::Ltr) => quote! {
      if let Some(path) = path.strip_prefix(#literal) { #children }
    },
    MatchInstructionKind::ConsumeLiteral(literal, MatchDirection::Rtl) => quote! {
      if let Some(path) = path.strip_suffix(#literal) { #children }
    },
    MatchInstructionKind::ConsumeLiteralInView(literal, MatchDirection::Ltr) => quote! {
      if let Some(view) = view.strip_prefix(#literal) { #children }
    },
    MatchInstructionKind::ConsumeLiteralInView(literal, MatchDirection::Rtl) => quote! {
      if let Some(view) = view.strip_suffix(#literal) { #children }
    },
    MatchInstructionKind::CheckEndOfPath => quote! {
      if path.is_empty() { #children }
    },
    MatchInstructionKind::ProcessRouteTargetMatch(segment_id) => {
      let segment = &ctx.routary.segment_map[segment_id];
      gen_segment_responder(ctx, segment)
    }
    MatchInstructionKind::ProcessNotFoundTargetMatch(segment_id) => quote! {
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
    Arity::Range(0, Some(max)) => {
      quote! { if segment.len() > #max { matched = false; break; } }
    }
    Arity::Range(min, None) => quote! { if segment.len() < #min { matched = false; break; } },
    Arity::Range(min, Some(max)) => {
      quote! { if segment.len() < #min || segment.len() > #max { matched = false; break; } }
    }
  }
}
