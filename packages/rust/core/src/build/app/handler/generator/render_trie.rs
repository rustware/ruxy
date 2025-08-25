use proc_macro2::TokenStream;
use quote::quote;

use crate::routing::instruction::MatchDirection;
use crate::util::radix_trie::{RadixTrie, RadixTrieNode};

/// Renders the provided RadixTrie into a prefix matching code.
///
/// If `reversed` is set to true, the rendered conditions will match
/// suffixes instead of prefixes.
pub fn render_trie(trie: &RadixTrie<TokenStream>, direction: MatchDirection) -> TokenStream {
  let items = trie.to_nodes().iter().map(|node| match node {
    RadixTrieNode::Item(item) => quote! { #item },
    RadixTrieNode::Prefix(prefix, subtrie) => {
      let subtrie = render_trie(subtrie, direction);

      let (strip_fn, literal) = match direction {
        MatchDirection::Ltr => (quote! { path.strip_prefix }, prefix),
        // Suffixes for right-to-left matching are inserted with its characters reversed.
        // Right-to-left matching is used for the part of the URL after a range-segment sequence.
        MatchDirection::Rtl => (quote! { path.strip_suffix }, &prefix.chars().rev().collect::<String>()),
      };

      quote! {
        if let Some(path) = #strip_fn(#literal) {
          #subtrie
        }
      }
    }
  });

  quote! { #(#items)* }
}
