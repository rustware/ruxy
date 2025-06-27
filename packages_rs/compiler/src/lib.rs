/// BIR (Build-time Intermediate Representation) is an enriched version of user's client code.
/// It's a valid executable JavaScript enriched with a lot of Ruxy-internal wrappers, symbols,
/// and other metadata as well as additional code responsible for generating RIR on execution.
mod bir;

mod compile_route;

pub use compile_route::compile_route;
