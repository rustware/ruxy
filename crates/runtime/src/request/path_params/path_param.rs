/// A PathParam holds the URL value of a matched Dynamic Sequence.
///
/// It exposes the raw value of one or multiple matched URL segments,
/// as well as methods to create an iterator over them.
pub struct PathParam<'raw> {
  allows_zero: bool,
  
  /// Raw path parameter value, as seen in the URL.
  ///
  /// The exact format of the value depends on how the Dynamic Sequence is defined:
  /// - For Dynamic Sequences with _Exact_ Segment Count (e.g. `{foo}` or `{foo[2]}`), the value is the exact string as
  ///   seen in the matched URL segment, with multiple matched segments separated by a slash (e.g. `foo` or `foo/bar`).
  /// - For Dynamic Sequences with _Range_ Segment Count, this further depends on the minimum allowed URL segments:
  ///   - For _required_ parameters (e.g. Dynamic Sequences that allow 1 or more segments to be matched, like `{foo[1..]}`
  ///     or `{foo[2..3]}`), the value is the exact string as seen in the matched URL segment, with multiple matched
  ///     segments separated by a Slash (e.g. `foo` or `foo/bar/baz`). This is the same as for _Exact_ Segment Count.
  ///   - For _optional_ parameters (e.g. Dynamic Sequences that allow 0 segments to be matched, like `{foo[0..]}` or
  ///     `{foo[0..1]}`), the leading slash is included before each segment's value, including the first one (e.g.
  ///     `/foo` of `/foo/bar`).
  ///     If such Dynamic Sequence is matched with 0 segments present in the URL, this value is an empty string.
  /// 
  /// The above distinctions are needed to differentiate between Empty Segment and 0 segments matched when
  /// the parameter captures zero or more URL segments. In such a case, empty string means 0 URL segments are matched,
  /// while `/` means that a single Empty Segment has been matched.
  pub raw: &'raw str,
}

impl<'raw> PathParam<'raw> {
  pub(crate) fn new(raw: &'raw str, allows_zero: bool) -> Self {
    Self {
      allows_zero,
      raw,
    }
  }
  
  /// Creates an iterator over the matched URL segments of this path parameter.
  /// 
  /// If your Dynamic Sequence only targets a single segment (e.g. `{foo}` or `{foo[1]}`),
  /// the iterator will return a single item.
  pub fn iter(&self) -> PathParamIter<'_> {
    PathParamIter::new(self.raw, self.allows_zero)
  }
}

pub struct PathParamIter<'raw> {
  enforce_zero: bool,
  split_iter: std::str::Split<'raw, char>,
}

impl<'raw> PathParamIter<'raw> {
  pub fn new(raw: &'raw str, allows_zero: bool) -> Self {
    let iter_str = if allows_zero { raw.strip_prefix('/').unwrap_or("") } else { raw };
    
    Self {
      enforce_zero: allows_zero && raw.is_empty(),
      split_iter: iter_str.split('/'),
    }
  }
}

impl<'raw> Iterator for PathParamIter<'raw> {
  type Item = &'raw str;

  fn next(&mut self) -> Option<Self::Item> {
    if self.enforce_zero {
      return None;
    }
    
    self.split_iter.next()
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    if self.enforce_zero {
      return (0, Some(0));
    }
    
    self.split_iter.size_hint()
  }
}

impl<'raw> DoubleEndedIterator for PathParamIter<'raw> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.enforce_zero {
      return None;
    }

    self.split_iter.next_back()
  }
}
