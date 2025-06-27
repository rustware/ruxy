use ruxy::{redirect, Caught, Loadable};

struct MyError {
  my_data: (String, String)
}

struct MyErr {
  my_attr: String,
}

struct MyOtherErr {
  my_other_attr: String,
}

async fn error_loader(mut error: impl Caught, /* req. extractors... */) -> impl Loadable {
  if let Some(err) = error.get_error::<MyErr>() {
    println!("Caught MyErr: {}", err.my_attr);
  }

  if let Some(err) = error.get_error_mut::<MyErr>() {
    err.my_attr.push('!');
    println!("Caught MyErr: {}", err.my_attr);
  }

  if let Some(mut err) = error.take_error::<MyErr>() {
    err.my_attr = "".to_string();
    println!("Caught MyErr: {}", err.my_attr);
  }

  if let Some(MyOtherErr { my_other_attr }) = error.get_error() {
    println!("Caught MyOtherErr: {my_other_attr}");
  }

  if let Some(MyOtherErr { my_other_attr }) = error.get_error_mut() {
    my_other_attr.push('!');
    println!("Caught MyOtherErr: {my_other_attr}");
  }

  if let Some(MyOtherErr { mut my_other_attr }) = error.take_error() {
    my_other_attr.push('!');
    println!("Caught MyOtherErr: {my_other_attr}");
  }

  redirect("/somewhere").permanent()
}
