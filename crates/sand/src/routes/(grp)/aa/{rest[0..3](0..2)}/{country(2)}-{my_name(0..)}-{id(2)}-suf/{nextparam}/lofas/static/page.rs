#[derive(ruxy::Props)]
pub struct Props {
  some_prop: String,
}

// For static pages (build-time props)
#[ruxy::generator]
pub async fn generate() -> ruxy::GeneratorOutput<Props> {
  // Create build-time props here
  ruxy::redirect("/somewhere").permanent().into()
}
