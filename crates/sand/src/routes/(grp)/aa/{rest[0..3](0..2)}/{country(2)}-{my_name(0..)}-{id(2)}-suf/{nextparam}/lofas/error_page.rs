use ruxy::{Caught, Loader};

struct MyError {
  my_data: (String, String)
}

pub fn error_loader(caught: impl Caught, /* req. extractors... */) -> impl Loader {
  if let Some(MyError { my_data }) = caught.get_error() {
    println!("Caught MyError: {my_data:?}");
  }
  
  // TODO: Introduce `match_error! {}` helper
}
