/// This is enough to represent every value the user can return from handlers.
/// `Vec` here doesn't mean a Vector returned by the user, but rather anything
/// we don't know the static count of ahead.
/// 
/// We're using this enum simply to be able to allocate the response buffer on
/// stack, where we only hold pointers to underlying data (be it String, &str,
/// Vec<Bytes>, and similar).
pub enum ResponseChunk {
  Bytes(bytes::Bytes),
  Vec(Vec<ResponseChunk>),
}
