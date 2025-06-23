use std::error::Error;

#[derive(ruxy::Props)]
pub struct Props {
  some_prop: String,
}

// For dynamic pages (server-side props)
#[ruxy::loader]
pub async fn load() -> ruxy::LoaderResult<Props> {
  // Testing all things a user can return here
  
  let a = k()?;
  
  if a == 0 {
    let lr: ruxy::LoaderResult<Props> = ruxy::redirect("/somewhere").permanent().into();
    return ruxy::redirect("/somewhere").permanent().into();
  }
  
  if a == 1 {
    return Ok(ruxy::redirect("/somewhere").permanent().into());
  }
  
  // if a == 3 {
  //   Ok(Props {
  //     some_prop: "some_prop".to_string(),
  //   })
  // }
  
  Err(MyE {}.into())
}

fn k() -> Result<u32, MyE> {
  Ok(0)
}

#[derive(Debug)]
struct MyE {}

impl std::fmt::Display for MyE {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "MyE")
  }
}

impl Error for MyE {}
