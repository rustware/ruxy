use proc_macro2::TokenStream;

use ::ruxy_util::radix_trie::RadixTrie;
use ::ruxy_routing::RouteSequence;

use crate::app::handler::router::context::GenContext;

type Trie = RadixTrie<TokenStream>;

pub fn with_custom_match(_ctx: &GenContext, sequence: &RouteSequence, subtrie: Trie) -> Trie {
  // TODO
  subtrie
}