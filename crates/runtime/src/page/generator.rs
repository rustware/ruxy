use crate::page::Props;
use crate::redirect::Redirect;

pub struct GeneratorOutput<P: Props = ()> {
  data: GeneratorData<P>,
}

enum GeneratorData<P: Props> {
  Props(P),
  Redirect(Redirect),
}

impl<P: Props> From<Redirect> for GeneratorOutput<P> {
  fn from(value: Redirect) -> Self {
    GeneratorOutput { data: GeneratorData::Redirect(value) }
  }
}
