use proc_macro::TokenStream;

use ::ruxy_routing::Arity;
use ::ruxy_util::radix_trie::RadixTrie;


pub struct TrieItem {
  /// Rendered matching logic
  pub tokens: TokenStream,
  /// Declares how many URL segments this rendered matching logic matches.
  ///
  /// This Arity specifier contains accumulated bounds, which is a sum of
  /// all lower bounds and sum of all upper bounds.
  ///
  /// If one of the contained Route Segments matches unlimited upper bound
  /// (e.g. `[n..]`), then this Arity specifier also has unlimited upper
  /// bound (`Arity::Range(n, None)`).
  /// 
  /// If all of the contained Route Segments specify an `Exact` count of
  /// matched URL segments, then this Arity specifier contains an `Exact`
  /// variant holding the sum of all contained `Exact` variant values.
  /// 
  /// Examples:
  /// `/{_[2]}/{_[3]}` => `Arity::Exact(5)`
  /// `/{_[2..4]}/{_[3]}` => `Arity::Range(5, 7)`
  /// `/{_[2..]}/{_[3]}` => `Arity::Range(5, None)`
  pub seg_count_acc: Arity,
}

pub type Trie = RadixTrie<TrieItem>;
