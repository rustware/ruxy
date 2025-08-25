mod loader_call;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::routing::routary::Routary;
use crate::routing::segment::{DynamicSequence, EitherTarget, HandlerTarget, RenderTarget, RouteSegment, SegmentEffect, TypedSequence};

use crate::build::app::context::GenContext;
use crate::build::app::handler::responder::loader_call::gen_loader_call;
use crate::build::build_config::BuildMode;

pub fn gen_segment_responder(ctx: &GenContext, segment: &RouteSegment) -> TokenStream {
  let identifier = &segment.identifier;

  let path_params: Vec<TokenStream> = extract_path_params(segment, ctx.routary);

  // TODO: If `page.tsx` exists, we'll return a page that says "Building...", with a websocket
  //       that connects to the server and updates the page as the build progresses.
  //       If the `page.tsx` doesn't exist, we'll return a page that outputs the route name,
  //       all the params and returned Props (if page.rs exists), and instructions on how to
  //       create the client page.

  let responder = match (&segment.route_target, &ctx.build_config.mode) {
    (Some(EitherTarget::Render(target)), BuildMode::Development) => gen_page_responder_dev(ctx, segment, target),
    (Some(EitherTarget::Render(target)), BuildMode::Production) => gen_page_responder_prod(ctx, segment, target),
    (Some(EitherTarget::Handler(target)), _) => gen_handler_responder(ctx, segment, target),
    _ => unreachable!("responder generator called for segment without a route target")
  };

  quote! {
    #responder

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

fn extract_path_params(segment: &RouteSegment, routes: &Routary) -> Vec<TokenStream> {
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

  if let Some(parent) = &segment.parent && let Some(parent) = routes.segment_map.get(parent) {
    v.extend(extract_path_params(parent, routes))
  }

  v
}

fn gen_page_responder_dev(ctx: &GenContext, segment: &RouteSegment, target: &RenderTarget) -> TokenStream {
  // TODO: Layout loader calls
  let loader_call = gen_loader_call(ctx, segment, target);

  quote! {
    #loader_call
  }
}

fn gen_page_responder_prod(ctx: &GenContext, segment: &RouteSegment, target: &RenderTarget) -> TokenStream {
  // TODO: Call pre-generated function on App::segment_blahblah_page();
  // TODO: Pre-generate that function.
  quote! {}
}

fn gen_handler_responder(ctx: &GenContext, segment: &RouteSegment, target: &HandlerTarget) -> TokenStream {
  quote! {}
}
