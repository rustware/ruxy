use proc_macro2::TokenStream;
use quote::quote;

use ::ruxy_util::radix_trie::{RadixTrie, RadixTrieNode};

/// Renders the provided RadixTrie into a prefix matching code.
/// 
/// If `reversed` is set to true, the rendered conditions will match
/// suffixes instead of prefixes.
pub fn render_trie(trie: &RadixTrie<TokenStream>, reversed: bool) -> TokenStream {
  let items = trie.to_nodes().iter().map(|node| match node {
    RadixTrieNode::Item(item) => quote! { #item },
    RadixTrieNode::Prefix(prefix, subtrie) => {
      let subtrie = render_trie(subtrie, reversed);

      let rhs = match reversed {
        false => quote! { path.strip_prefix },
        // Suffixes for end-to-start matching are inserted with reversed characters.
        // End-to-start matching is used for the part of the URL after a range-segment sequence.
        true => quote! { path.strip_suffix },
      };
      
      quote! {
        if let Some(path) = #rhs(#prefix) {
          #subtrie
        }
      }
    }
  });

  quote! { #(#items)* }
}
