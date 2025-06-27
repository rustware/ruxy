use ruxy::Loadable;

#[derive(ruxy::Props)]
pub struct Props {
  some_prop: String,
}

impl Loader for super::Route {
  async fn load() -> String {
    "".to_string()
  }
}



fn to_be_cached() {
  
}

// --------------------------------

struct Cache {}

trait Loader {
  async fn load() -> impl Loadable;
}

// For dynamic pages (server-side props)
pub async fn loader() -> impl Loadable {
  // Testing all things the user can return here

  let a = k()?;

  if a == 0 {
    let _: ruxy::LoaderOutput<Props> = ruxy::redirect("/somewhere").permanent().into();
    let _: Result<ruxy::LoaderOutput, String> = ruxy::redirect("/somewhere").permanent().into();
    return ruxy::redirect("/somewhere").permanent().into();
  }

  if a == 1 {
    return Ok(ruxy::redirect("/somewhere").permanent().into());
  }

  if a == 3 {
    let props = Props { some_prop: "some_prop".to_string() };

    return Ok(props.into());
  }

  if a == 4 {
    let props = Props { some_prop: "some_prop".to_string() };

    // Doesn't work yet (is it even possible?)
    return props.into();
  }

  Err(String::new())
}

fn k() -> Result<u8, String> {
  Ok(0)
}

// TODO: How to validate the loadeer function signature?

trait Validator {
  fn a() {}
}
