pub trait Props {}

// TODO: Macro-generate these

impl Props for bool {}

impl Props for i8 {}
impl Props for i16 {}
impl Props for i32 {}
impl Props for i64 {}
impl Props for i128 {}
impl Props for isize {}

impl Props for u8 {}
impl Props for u16 {}
impl Props for u32 {}
impl Props for u64 {}
impl Props for u128 {}
impl Props for usize {}

impl Props for f32 {}
impl Props for f64 {}

impl Props for String {}
impl Props for str {}

impl Props for () {}

impl<T> Props for &T {}

impl<T> Props for [T] {}
impl<T> Props for Vec<T> {}
