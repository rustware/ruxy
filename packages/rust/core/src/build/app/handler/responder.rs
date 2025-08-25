use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::routing::routary::Routary;
use crate::routing::segment::{DynamicSequence, RouteSegment, SegmentEffect, TypedSequence};

use crate::build::app::handler::generator::context::GenContext;

pub fn gen_segment_responder(ctx: &GenContext, segment: &RouteSegment) -> TokenStream {
  let identifier = &segment.identifier;

  let path_params: Vec<TokenStream> = extract_idents_for_segment(segment, ctx.routes);

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

fn extract_idents_for_segment(segment: &RouteSegment, routes: &Routary) -> Vec<TokenStream> {
  let mut v = Vec::new();

  if let SegmentEffect::UrlMatcher { sequences } = &segment.effect {
    let param_names = sequences.iter().filter_map(|s| {
      let TypedSequence::Dynamic(DynamicSequence { param_name, .. }) = &s.typed else {
        return None;
      };

      Some(param_name)
    });

    for param_name in param_names {
      let dyn_var_ident = format!("path_param_{param_name}");
      let dyn_var_ident = Ident::new(&dyn_var_ident, Span::mixed_site());

      v.push(quote! {
        body.push(internal::Bytes::from("<div>"));
        let formatted = format!("{}: {:?}", #param_name, #dyn_var_ident);
        body.push(internal::Bytes::from(formatted));
        body.push(internal::Bytes::from("</div>"));
      });
    }
  }

  if let Some(parent) = &segment.parent {
    if let Some(parent) = routes.segment_map.get(parent) {
      v.extend(extract_idents_for_segment(parent, routes))
    }
  }

  v
}
