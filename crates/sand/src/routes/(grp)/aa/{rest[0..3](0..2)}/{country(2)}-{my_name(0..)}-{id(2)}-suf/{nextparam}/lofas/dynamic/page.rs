use std::sync::OnceLock;

#[derive(ruxy::Props)]
pub struct Props {
  some_prop: String,
}

// For dynamic pages (server-side props)
#[ruxy::loader]
pub async fn load() -> Result<ruxy::LoaderOutput<Props>, MyErr> {
  // Testing all things the user can return here

  let a = k()?;

  if a == 0 {
    let _: ruxy::LoaderOutput<Props> = ruxy::redirect("/somewhere").permanent().into();
    let _: Result<ruxy::LoaderOutput, MyErr> = ruxy::redirect("/somewhere").permanent().into();
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

  Err(MyErr {})
}

fn k() -> Result<u32, MyErr> {
  Ok(0)
}

#[derive(Debug)]
pub struct MyErr {}

type ReturnAlias = Result<String, std::io::Error>;

trait ExtractError {
  type Error;
}

// This allows the macro to extract the concrete error type from the return type
// even when it's behind a type alias.
impl<T: ruxy::Props, E> ExtractError for Result<T, E> {
  type Error = E;
}

type MyError = <ReturnAlias as ExtractError>::Error;

// Of course we'll need to implement the ExtractError trait
// for all return types (even infallible ones; setting some dummy type for them)

// App macro will validate that all error types are the same, using a const function:
// const fn validate_same_error_types() {
//   / nope, `TypeId::of::<T>()` cannot be called in const contexts...
// }
