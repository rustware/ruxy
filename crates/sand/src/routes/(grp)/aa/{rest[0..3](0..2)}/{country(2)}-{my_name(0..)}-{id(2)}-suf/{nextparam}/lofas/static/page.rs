#[derive(ruxy::Props)]
pub struct Props {
  some_prop: String,
}

// This will be emitted by the `ruxy::loader` macro:
// -- start --
pub type MyErrorB = (String, String);

#[macro_export] macro_rules! __ruxy_user_error_variant_modulehex2_page {
  () => { MyErrorB(MyErrorB), };
}
// --- end ---

// For static pages (build-time props)
#[ruxy::generator]
pub async fn generate() -> ruxy::GeneratorOutput<Props> {
  // Create build-time props here
  ruxy::redirect("/somewhere").permanent().into()
}
