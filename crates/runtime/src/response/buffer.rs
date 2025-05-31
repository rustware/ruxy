use std::mem::MaybeUninit;

use super::chunk::ResponseChunk;

/// Ruxy internal response body buffer.
/// 
/// A stack-allocated, fixed-capacity array that supports partial initialization,
/// up to the limit of 300 kilobytes, where it switches to heap allocation.
///
/// `ReponseBuffer` provides a way to incrementally construct an array of `Chunk`s
/// without requiring heap allocation up to 300 kB. Internally, it uses `MaybeUninit`
/// to allow storing uninitialized chunks with low overhead, and tracks how many
/// chunks have been initialized via the `head` counter.
///
/// There is a second counter called `tail`, which tracks how many chunks have
/// been consumed from the array. Both writing and consuming starts at index 0.
///
/// This buffer cannot be reused – it's not a ring buffer. Both the head and tail
/// indices can only progress in one direction, and don't wrap around.
///
/// This is particularly useful in performance-critical code where you want to
/// avoid dynamic allocation and prefer precise control over memory initialization.
///
/// IMPORTANT: `ResponseBuffer` will allocate on stack up to the limit of 300 kilobytes.
/// If the maximum capacity exceeds this limit, `ResponseBuffer` will allocate on
/// the heap to prevent stack overflows.
pub struct ResponseBuffer<const CAP: usize> {
  items: [MaybeUninit<ResponseChunk>; CAP],
  head: usize,
  tail: usize,
}

impl<const CAP: usize> ResponseBuffer<CAP> {
  // Thread stack limit is anywhere between 1 to 8 MBs depending on the platform, we'll take
  // the conservative approach and use less than 3rd of the smallest stack limit.
  const MAX_STACK_SIZE_BYTES: usize = 330_000;
  const MAX_STACK_SIZE_COUNT: usize = Self::MAX_STACK_SIZE_BYTES / size_of::<ResponseChunk>();  
  const MAX_STACK_CHUNKS: bool = size_of::<ResponseChunk>().saturating_mul(CAP) > 300_000;
  const UNINIT: MaybeUninit<ResponseChunk> = MaybeUninit::uninit();

  /// Constructs a new `PartialArray` with the given `CAP` capacity.
  pub fn new() -> Self {
    Self { items: [Self::UNINIT; CAP], head: 0, tail: 0 }
  }

  /// Write an item to the head of the array.
  /// This function will panic if the head is at the array's capacity.
  pub fn push(&mut self, item: ResponseChunk) {
    self.items[self.head] = MaybeUninit::new(item);
    self.head += 1;
  }

  /// Write an item to the head of the array.
  /// This function will NOT check that the head is below the array's capacity.
  ///
  /// ### Safety
  /// The caller must guarantee the head is below the array's capacity.
  /// Pushing an item to an array which has its head reached capacity is Undefined Behavior.
  pub unsafe fn push_unchecked(&mut self, item: ResponseChunk) {
    // SAFETY: the caller must uphold the documented safety contract.
    unsafe { self.items.get_unchecked_mut(self.head) }.write(item);
    self.head += 1;
  }

  /// Consumes an item from the array and moves its tail cursor.
  /// This function will return None if the tail is not below the head.
  pub fn consume(&mut self) -> Option<ResponseChunk> {
    // Check we're operating inside safe bounds
    if self.tail >= self.head {
      return None;
    }

    // SAFETY: we just checked that the tail is below the head
    Some(unsafe { self.consume_unchecked() })
  }

  /// Consumes an item from the array and moves its tail cursor.
  /// This function does NOT check that the tail is below the head.
  ///
  /// ### Safety
  /// The caller must guarantee the tail is below the head.
  /// Pushing an item to an array with `tail >= head` is Undefined Behavior.
  pub unsafe fn consume_unchecked(&mut self) -> ResponseChunk {
    // Replace the item with uninitialized memory
    let item = std::mem::replace(&mut self.items[self.tail], MaybeUninit::uninit());

    // Increment the tail cursor
    self.tail += 1;

    // SAFETY: we guarantee the area between `self.tail` and `self.head` is initialized,
    //         the caller must uphold the documented safety contract and ensure we're
    //         operating inside these safe bounds.
    unsafe { item.assume_init() }
  }

  pub fn is_consumed(&self) -> bool {
    self.tail == self.head
  }

  pub fn unconsumed_size(&self) -> u64 {
    (self.head - self.tail) as u64
  }
}

impl<const CAP: usize> Drop for ResponseBuffer<CAP> {
  fn drop(&mut self) {
    for item in &mut self.items[self.tail..self.head] {
      // SAFETY: we guarantee the area between `self.tail` and `self.head` is initialized.
      unsafe { item.assume_init_drop() }
    }
  }
}

impl<const CAP: usize> std::ops::Deref for ResponseBuffer<CAP> {
  type Target = [ResponseChunk];

  fn deref(&self) -> &Self::Target {
    let filled = &self.items[self.tail..self.head];

    // SAFETY: 1. we guarantee the area between `self.tail` and `self.head` is initialized.
    //         2. `MaybeUninit` is `#[repr(transparent)]`, transmute is safe.
    unsafe { std::mem::transmute(filled) }
  }
}

impl<const CAP: usize> std::ops::DerefMut for ResponseBuffer<CAP> {
  /// Dereference to the slice of filled elements (potentially less than `N`).
  fn deref_mut(&mut self) -> &mut Self::Target {
    let filled = &mut self.items[self.tail..self.head];

    // SAFETY: 1. we guarantee the area between `self.tail` and `self.head` is initialized.
    //         2. `MaybeUninit` is `#[repr(transparent)]`, transmute is safe.
    unsafe { std::mem::transmute(filled) }
  }
}
