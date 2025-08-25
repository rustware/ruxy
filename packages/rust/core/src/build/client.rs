use crate::build::BuildConfig;
use crate::routing::routary::Routary;
use crate::routing::segment::RouteSegmentClientEntry;

// TODO: emit watch hints for all client files discovered during the build process

/// Builds the whole client application (all pages, layouts, etc.) for Production.
pub(crate) fn build_all(build_config: &BuildConfig, routary: &Routary) {
  // TODO: Walk over the Routary and call `build_route_segment_entries` with all the entries.
}


/// Builds the requested route segment entry (Development mode).
pub fn build_route_segment_entries(build_config: &BuildConfig, entries: Vec<&RouteSegmentClientEntry>) {

}
